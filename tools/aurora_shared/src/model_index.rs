//! In-memory full-text indexing over Aurora card files with live filesystem updates.

use std::collections::BTreeSet;
use std::path::{Component, Path, PathBuf};
use std::sync::{Arc, Mutex};

use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use serde_json::Value;
use tantivy::collector::TopDocs;
use tantivy::query::{QueryParser, TermQuery};
use tantivy::schema::{Field, IndexRecordOption, STORED, STRING, Schema, TEXT, Value as _};
use tantivy::{Index, IndexReader, IndexWriter, TantivyDocument, Term};
use thiserror::Error;
use tracing::warn;
use walkdir::WalkDir;

const INDEX_WRITER_HEAP_BYTES: usize = 50_000_000;
const SEARCH_LIMIT: usize = 50;

/// Search result returned by [`ModelIndex::search`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CardRef {
	pub id: String,
	pub card_type: String,
	pub card_subtype: Option<String>,
	pub name: String,
}

/// In-memory Aurora card index kept fresh via filesystem notifications.
pub struct ModelIndex {
	state: Arc<ModelIndexState>,
	_watcher: RecommendedWatcher,
}

impl ModelIndex {
	/// Build the in-memory index and start watching the model home for card changes.
	pub fn open(path: &Path) -> Result<Self, ModelIndexError> {
		let model_home = resolve_model_home(path)?;
		let fields = IndexFields::new();
		let schema = fields.schema();
		let index = Index::create_in_ram(schema);
		let reader = index.reader()?;
		let writer = index.writer(INDEX_WRITER_HEAP_BYTES)?;
		let state = Arc::new(ModelIndexState {
			model_home: model_home.clone(),
			index,
			reader,
			writer: Mutex::new(writer),
			fields,
		});

		state.rebuild()?;

		let watcher_state = Arc::clone(&state);
		let mut watcher = notify::recommended_watcher(move |result| match result {
			Ok(event) => {
				if let Err(error) = watcher_state.apply_event(event) {
					warn!(error = %error, "Failed to update ModelIndex from filesystem event");
				}
			}
			Err(error) => warn!(error = %error, "Filesystem watcher error for ModelIndex"),
		})?;
		watcher.watch(&model_home, RecursiveMode::Recursive)?;

		Ok(Self {
			state,
			_watcher: watcher,
		})
	}

	/// Search the indexed Aurora cards.
	pub fn search(&self, query: &str) -> Result<Vec<CardRef>, ModelIndexError> {
		self.state.search(query)
	}

	/// Resolve the model-home-relative file path for a card ID using the index.
	pub fn resolve_card_path(&self, id: &str) -> Result<Option<PathBuf>, ModelIndexError> {
		self.state.resolve_card_path(id)
	}

	/// Resolve cards that link directly to the provided target card ID.
	pub fn find_cards_linking_to(&self, id: &str) -> Result<Vec<CardRef>, ModelIndexError> {
		self.state.find_cards_linking_to(id)
	}
}

struct ModelIndexState {
	model_home: PathBuf,
	index: Index,
	reader: IndexReader,
	writer: Mutex<IndexWriter>,
	fields: IndexFields,
}

impl ModelIndexState {
	fn rebuild(&self) -> Result<(), ModelIndexError> {
		let mut writer = self
			.writer
			.lock()
			.map_err(|_| ModelIndexError::WriterPoisoned)?;
		writer.delete_all_documents()?;
		for path in card_files(&self.model_home)? {
			if let Some(document) = self.build_document_for_path(path.as_path())? {
				writer.add_document(document)?;
			}
		}
		writer.commit()?;
		self.reader.reload()?;
		Ok(())
	}

	fn search(&self, query: &str) -> Result<Vec<CardRef>, ModelIndexError> {
		let trimmed = query.trim();
		if trimmed.is_empty() {
			return Ok(Vec::new());
		}

		let parser = self.query_parser();
		let query = parser.parse_query(trimmed)?;
		let searcher = self.reader.searcher();
		let top_docs = searcher.search(&query, &TopDocs::with_limit(SEARCH_LIMIT))?;

		top_docs
			.into_iter()
			.map(|(_, address)| {
				let document: TantivyDocument = searcher.doc(address)?;
				self.card_ref_from_document(&document)
			})
			.collect()
	}

	fn resolve_card_path(&self, id: &str) -> Result<Option<PathBuf>, ModelIndexError> {
		let searcher = self.reader.searcher();
		let query = TermQuery::new(
			Term::from_field_text(self.fields.id, id),
			IndexRecordOption::Basic,
		);
		let top_docs = searcher.search(&query, &TopDocs::with_limit(1))?;
		let Some((_, address)) = top_docs.into_iter().next() else {
			return Ok(None);
		};

		let document: TantivyDocument = searcher.doc(address)?;
		let path = first_text(&document, self.fields.path)
			.ok_or(ModelIndexError::MissingStoredField("path"))?;
		Ok(Some(PathBuf::from(path)))
	}

	fn find_cards_linking_to(&self, id: &str) -> Result<Vec<CardRef>, ModelIndexError> {
		let searcher = self.reader.searcher();
		let query = TermQuery::new(
			Term::from_field_text(self.fields.link_targets_exact, id),
			IndexRecordOption::Basic,
		);
		let top_docs = searcher.search(&query, &TopDocs::with_limit(SEARCH_LIMIT))?;
		let mut refs = top_docs
			.into_iter()
			.map(|(_, address)| {
				let document: TantivyDocument = searcher.doc(address)?;
				self.card_ref_from_document(&document)
			})
			.collect::<Result<Vec<_>, _>>()?;
		refs.sort_by(|left, right| left.id.cmp(&right.id));
		Ok(refs)
	}

	fn apply_event(&self, event: Event) -> Result<(), ModelIndexError> {
		if !matches!(
			event.kind,
			EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_)
		) {
			return Ok(());
		}

		let mut writer = self
			.writer
			.lock()
			.map_err(|_| ModelIndexError::WriterPoisoned)?;
		let mut changed = false;
		for path in event.paths {
			if !is_card_path(path.as_path(), &self.model_home) {
				continue;
			}

			let path_key = relative_path_key(&self.model_home, path.as_path())?;
			writer.delete_term(Term::from_field_text(self.fields.path, path_key.as_str()));
			changed = true;

			if path.is_file()
				&& let Some(document) = self.build_document_for_path(path.as_path())?
			{
				writer.add_document(document)?;
			}
		}

		if changed {
			writer.commit()?;
			self.reader.reload()?;
		}

		Ok(())
	}

	fn build_document_for_path(
		&self,
		path: &Path,
	) -> Result<Option<TantivyDocument>, ModelIndexError> {
		if !is_card_path(path, &self.model_home) {
			return Ok(None);
		}

		let raw = std::fs::read_to_string(path)?;
		let json: Value = serde_json::from_str(&raw)
			.map_err(|error| ModelIndexError::CardParse(path.display().to_string(), error))?;
		let card = IndexedCard::from_json(path, &json)?;
		Ok(Some(self.card_document(path, &card)?))
	}

	fn card_document(
		&self,
		path: &Path,
		card: &IndexedCard,
	) -> Result<TantivyDocument, ModelIndexError> {
		let mut document = TantivyDocument::default();
		document.add_text(
			self.fields.path,
			relative_path_key(&self.model_home, path)?.as_str(),
		);
		document.add_text(self.fields.id, card.id.as_str());
		document.add_text(self.fields.card_type, card.card_type.as_str());
		if let Some(card_subtype) = &card.card_subtype {
			document.add_text(self.fields.card_subtype, card_subtype.as_str());
		}
		document.add_text(self.fields.name, card.name.as_str());
		for target in &card.link_targets {
			document.add_text(self.fields.link_targets, target.as_str());
			document.add_text(self.fields.link_targets_exact, target.as_str());
		}
		for attribute in &card.attribute_names {
			document.add_text(self.fields.attribute_names, attribute.as_str());
		}
		Ok(document)
	}

	fn card_ref_from_document(
		&self,
		document: &TantivyDocument,
	) -> Result<CardRef, ModelIndexError> {
		Ok(CardRef {
			id: first_text(document, self.fields.id)
				.ok_or(ModelIndexError::MissingStoredField("id"))?,
			card_type: first_text(document, self.fields.card_type)
				.ok_or(ModelIndexError::MissingStoredField("card_type"))?,
			card_subtype: first_text(document, self.fields.card_subtype),
			name: first_text(document, self.fields.name)
				.ok_or(ModelIndexError::MissingStoredField("name"))?,
		})
	}

	fn query_parser(&self) -> QueryParser {
		let mut parser = QueryParser::for_index(
			&self.index,
			vec![
				self.fields.id,
				self.fields.card_type,
				self.fields.card_subtype,
				self.fields.name,
				self.fields.link_targets,
				self.fields.attribute_names,
			],
		);
		parser.set_field_boost(self.fields.id, 6.0);
		parser.set_field_boost(self.fields.name, 5.0);
		parser.set_field_boost(self.fields.card_type, 3.0);
		parser.set_field_boost(self.fields.card_subtype, 2.0);
		parser.set_field_boost(self.fields.link_targets, 1.0);
		parser.set_field_boost(self.fields.attribute_names, 1.0);
		parser
	}
}

fn is_card_path(path: &Path, model_home: &Path) -> bool {
	if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
		return false;
	}
	if path.file_name().and_then(|name| name.to_str()) == Some("Compact.json") {
		return false;
	}

	let Ok(relative) = path.strip_prefix(model_home) else {
		return false;
	};
	let mut components = relative.components();
	match components.next() {
		Some(Component::Normal(first)) => !matches!(
			first.to_str(),
			Some("reference") | Some("schemas") | Some("backups")
		),
		_ => false,
	}
}

fn relative_path_key(model_home: &Path, path: &Path) -> Result<String, ModelIndexError> {
	let relative = path
		.strip_prefix(model_home)
		.map_err(|_| ModelIndexError::PathOutsideModelHome(path.display().to_string()))?;
	Ok(relative.to_string_lossy().replace('\\', "/"))
}

fn resolve_model_home(path: &Path) -> Result<PathBuf, ModelIndexError> {
	if path
		.file_name()
		.and_then(|name| name.to_str())
		.is_some_and(|name| name == "aurora")
		&& path.is_dir()
	{
		return Ok(path.to_path_buf());
	}

	let nested = path.join("aurora");
	if nested.is_dir() {
		return Ok(nested);
	}

	Err(ModelIndexError::InvalidModelHome(
		path.display().to_string(),
	))
}

fn card_files(model_home: &Path) -> Result<Vec<PathBuf>, ModelIndexError> {
	let mut files = Vec::new();
	for entry in WalkDir::new(model_home) {
		let entry = entry?;
		if entry.file_type().is_file() && is_card_path(entry.path(), model_home) {
			files.push(entry.into_path());
		}
	}
	files.sort();
	Ok(files)
}

fn first_text(document: &TantivyDocument, field: Field) -> Option<String> {
	document
		.get_first(field)
		.and_then(|value| value.as_str().map(ToString::to_string))
}

fn required_string(json: &Value, field: &str, path: &Path) -> Result<String, ModelIndexError> {
	json.get(field)
		.and_then(Value::as_str)
		.map(ToString::to_string)
		.ok_or_else(|| ModelIndexError::MissingField(path.display().to_string(), field.to_string()))
}

fn optional_string(json: &Value, field: &str) -> Option<String> {
	json.get(field)
		.and_then(Value::as_str)
		.map(ToString::to_string)
}

#[derive(Clone)]
struct IndexFields {
	path: Field,
	id: Field,
	card_type: Field,
	card_subtype: Field,
	name: Field,
	link_targets: Field,
	link_targets_exact: Field,
	attribute_names: Field,
	schema: Schema,
}

impl IndexFields {
	fn new() -> Self {
		let mut schema_builder = Schema::builder();
		let path = schema_builder.add_text_field("path", STRING | STORED);
		let id = schema_builder.add_text_field("id", STRING | STORED);
		let card_type = schema_builder.add_text_field("card_type", TEXT | STORED);
		let card_subtype = schema_builder.add_text_field("card_subtype", TEXT | STORED);
		let name = schema_builder.add_text_field("name", TEXT | STORED);
		let link_targets = schema_builder.add_text_field("link_targets", TEXT);
		let link_targets_exact = schema_builder.add_text_field("link_targets_exact", STRING);
		let attribute_names = schema_builder.add_text_field("attribute_names", TEXT);
		let schema = schema_builder.build();
		Self {
			path,
			id,
			card_type,
			card_subtype,
			name,
			link_targets,
			link_targets_exact,
			attribute_names,
			schema,
		}
	}

	fn schema(&self) -> Schema {
		self.schema.clone()
	}
}

struct IndexedCard {
	id: String,
	card_type: String,
	card_subtype: Option<String>,
	name: String,
	link_targets: BTreeSet<String>,
	attribute_names: BTreeSet<String>,
}

impl IndexedCard {
	fn from_json(path: &Path, json: &Value) -> Result<Self, ModelIndexError> {
		let id = required_string(json, "id", path)?;
		let card_type = required_string(json, "card_type", path)?;
		let name = required_string(json, "name", path)?;
		let card_subtype = optional_string(json, "card_subtype");
		let link_targets = json
			.get("links")
			.and_then(Value::as_array)
			.map(|links| {
				links
					.iter()
					.filter_map(|link| link.get("target").and_then(Value::as_str))
					.map(ToString::to_string)
					.collect()
			})
			.unwrap_or_default();
		let attribute_names = json
			.get("attributes")
			.and_then(Value::as_object)
			.map(|attributes| attributes.keys().cloned().collect())
			.unwrap_or_default();
		Ok(Self {
			id,
			card_type,
			card_subtype,
			name,
			link_targets,
			attribute_names,
		})
	}
}

/// Errors raised while building, maintaining, or querying the Aurora model index.
#[derive(Debug, Error)]
pub enum ModelIndexError {
	#[error("Invalid Aurora model home for indexing: {0}")]
	InvalidModelHome(String),
	#[error("Card file is missing required field '{1}': {0}")]
	MissingField(String, String),
	#[error("Indexed document is missing stored field '{0}'")]
	MissingStoredField(&'static str),
	#[error("Card parse error at {0}: {1}")]
	CardParse(String, serde_json::Error),
	#[error("Indexed path is outside the model home: {0}")]
	PathOutsideModelHome(String),
	#[error("ModelIndex writer mutex was poisoned")]
	WriterPoisoned,
	#[error("ModelIndex I/O error: {0}")]
	Io(#[from] std::io::Error),
	#[error("ModelIndex search error: {0}")]
	Tantivy(#[from] tantivy::TantivyError),
	#[error("ModelIndex query parse error: {0}")]
	QueryParser(#[from] tantivy::query::QueryParserError),
	#[error("ModelIndex watcher error: {0}")]
	Notify(#[from] notify::Error),
	#[error("ModelIndex directory walk error: {0}")]
	Walkdir(#[from] walkdir::Error),
}

#[cfg(test)]
#[path = "model_index_tests.rs"]
mod tests;
