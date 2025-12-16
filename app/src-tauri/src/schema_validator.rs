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
        let card_types = vec![
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
            if let Ok(schema_text) = std::fs::read_to_string(&schema_path) {
                if let Ok(schema_value) = serde_json::from_str::<Value>(&schema_text) {
                    if let Ok(compiled_schema) = JSONSchema::compile(&schema_value) {
                        let ct = card_type.clone();
                        schemas.insert(card_type, compiled_schema);
                        log::debug!("Loaded schema for {:?}", ct);
                    }
                }
            }
        }

        if schemas.is_empty() {
            log::warn!("No schemas loaded from {:?}", schema_dir);
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
            Err(_e) => {
                let error_msg = "Card validation failed: JSON schema validation error";
                log::warn!("{}", error_msg);
                let err = AppError::validation("Card does not conform to schema", error_msg);
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
