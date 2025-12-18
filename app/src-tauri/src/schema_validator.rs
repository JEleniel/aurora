use crate::models::{AppError, Card, CardType};
use jsonschema::JSONSchema;
use serde_json::Value;
use std::collections::HashMap;
use std::path::Path;

pub struct SchemaValidator {
    schemas: HashMap<CardType, JSONSchema>,
}

impl SchemaValidator {
    pub fn new(schema_dir: &Path) -> Result<Self, String> {
        let mut schemas = HashMap::new();

        // First, load the base card schema for reference resolution
        let card_schema_path = schema_dir.join("card.schema.json");
        let base_schema = match std::fs::read_to_string(&card_schema_path) {
            Ok(schema_text) => match serde_json::from_str::<Value>(&schema_text) {
                Ok(schema_value) => {
                    log::debug!("Loaded base card schema");
                    Some(schema_value)
                }
                Err(e) => {
                    log::warn!("Failed to parse base card schema: {}", e);
                    None
                }
            },
            Err(e) => {
                log::debug!("Base card schema not found: {}", e);
                None
            }
        };

        let card_types = vec![
            (CardType::Mission, "mission.schema.json"),
            (CardType::Driver, "driver.schema.json"),
            (CardType::Requirement, "requirement.schema.json"),
            (CardType::Behavior, "behavior.schema.json"),
            (CardType::Interface, "interface.schema.json"),
            (CardType::Constraint, "constraint.schema.json"),
            (CardType::LogicalComponent, "logical-component.schema.json"),
            (CardType::DeployableNode, "deployable-node.schema.json"),
            (CardType::Actor, "actor.schema.json"),
            (CardType::Test, "test.schema.json"),
            (CardType::Artifact, "artifact.schema.json"),
            (CardType::View, "view.schema.json"),
            (CardType::Note, "note.schema.json"),
        ];

        for (card_type, filename) in card_types {
            let schema_path = schema_dir.join(filename);
            match std::fs::read_to_string(&schema_path) {
                Ok(schema_text) => match serde_json::from_str::<Value>(&schema_text) {
                    Ok(mut schema_value) => {
                        // Inline the base card schema to resolve $ref
                        if let Some(ref base) = base_schema {
                            if let Some(ref mut allof) = schema_value.get_mut("allOf") {
                                if let Some(allof_arr) = allof.as_array_mut() {
                                    for item in allof_arr.iter_mut() {
                                        if let Some(ref_str) = item.get("$ref") {
                                            if ref_str.as_str() == Some("card.schema.json") {
                                                // Replace $ref with inline schema
                                                *item = base.clone();
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        match JSONSchema::compile(&schema_value) {
                            Ok(compiled_schema) => {
                                let ct = card_type.clone();
                                schemas.insert(card_type, compiled_schema);
                                log::debug!("Loaded schema for {:?}", ct);
                            }
                            Err(e) => {
                                log::warn!("Failed to compile schema for {:?}: {}", card_type, e);
                            }
                        }
                    }
                    Err(e) => {
                        log::warn!("Failed to parse JSON schema {}: {}", filename, e);
                    }
                },
                Err(e) => {
                    log::debug!("Schema file not found: {:?} ({})", schema_path, e);
                }
            }
        }

        if schemas.is_empty() {
            log::warn!("No schemas loaded from {:?}", schema_dir);
            log::warn!("Some schemas may still load later, but validation will be skipped until schemas are available");
        } else {
            log::info!("Successfully loaded {} schemas", schemas.len());
        }

        Ok(SchemaValidator { schemas })
    }

    pub fn validate_card(&self, card: &Card) -> Result<(), String> {
        let schema = match self.schemas.get(&card.r#type) {
            Some(s) => s,
            None => {
                log::warn!(
                    "No schema available for card type {:?}, skipping validation",
                    card.r#type
                );
                return Ok(());
            }
        };

        let card_json = match serde_json::to_value(card) {
            Ok(v) => v,
            Err(e) => {
                let err = AppError::validation(
                    "Failed to serialize card for validation",
                    format!("Serialization error: {}", e),
                );
                return Err(String::from(err));
            }
        };

        match schema.validate(&card_json) {
            Ok(()) => {
                log::debug!("Card '{}' passed validation", card.name);
                Ok(())
            }
            Err(e) => {
                let validation_errors: Vec<String> = e
                    .collect::<Vec<_>>()
                    .iter()
                    .map(|err| format!("{}: {}", err.instance_path, err.to_string()))
                    .collect();
                let error_details = validation_errors.join("; ");
                log::warn!("Card '{}' validation failed: {}", card.name, error_details);
                let err = AppError::validation(
                    "Card does not conform to schema",
                    format!("Validation errors: {}", error_details),
                );
                Err(String::from(err))
            }
        }
    }

    pub fn get_required_fields(&self, card_type: CardType) -> Vec<String> {
        self.schemas
            .get(&card_type)
            .map(|_| vec!["name".to_string(), "description".to_string()])
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schema_validator_creation() {
        let validator = SchemaValidator::new(Path::new("/nonexistent"));
        assert!(validator.is_ok());
    }
}
