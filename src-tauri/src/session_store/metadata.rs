use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Current metadata schema version. Increment when the schema changes.
pub const SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SessionStatus {
    Active,
    Archived,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMetadata {
    pub schema_version: u32,
    pub id: String,
    pub created_at: String,
    pub status: SessionStatus,
    pub label: Option<String>,
}

impl SessionMetadata {
    pub fn new(label: Option<String>) -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            id: Uuid::new_v4().to_string(),
            created_at: Utc::now().to_rfc3339(),
            status: SessionStatus::Active,
            label,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metadata_roundtrip() {
        let meta = SessionMetadata::new(Some("test label".to_string()));
        let json = serde_json::to_string(&meta).unwrap();
        let deserialized: SessionMetadata = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.schema_version, SCHEMA_VERSION);
        assert_eq!(deserialized.id, meta.id);
        assert_eq!(deserialized.status, SessionStatus::Active);
        assert_eq!(deserialized.label, Some("test label".to_string()));
    }

    #[test]
    fn test_metadata_tolerates_extra_fields() {
        let json = r#"{
            "schema_version": 1,
            "id": "abc-123",
            "created_at": "2026-03-30T00:00:00Z",
            "status": "active",
            "label": null,
            "future_field": "should be ignored"
        }"#;

        // serde_json ignores unknown fields by default
        let meta: SessionMetadata = serde_json::from_str(json).unwrap();
        assert_eq!(meta.id, "abc-123");
        assert_eq!(meta.status, SessionStatus::Active);
    }

    #[test]
    fn test_schema_version_is_current() {
        let meta = SessionMetadata::new(None);
        assert_eq!(meta.schema_version, 1);
    }
}
