// SPDX-FileCopyrightText: Copyright Logically Magic Inc. <hello@logicallymagic.com>
// SPDX-FileCopyrightText: Copyright Hamid Jahanzad <Hamid@Jahanzad.com>
// SPDX-FileCopyrightText: Copyright Contributors to the Bank8:SDK
// SPDX-License-Identifier: Apache-2.0

//! # bank8-fdx-contracts
//!
//! Wire types for the Bank8:FDX open-banking HTTP API (FDX v6.5). See the
//! crate `README.md` for the design overview.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// An FDX consent grant as returned by the consent reads.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Consent {
    /// Consent id (globally unique).
    pub id: Uuid,
    /// Owning tenant environment.
    pub env_id: Uuid,
    /// Lifecycle state (`ACTIVE`, `REVOKED`, …).
    pub status: String,
    /// Granted FDX permission strings (e.g. `accounts:read`).
    pub scopes: Vec<String>,
    /// Grant timestamp.
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_consent() -> Consent {
        Consent {
            id: uuid::Uuid::parse_str("00000000-0000-0000-0000-00000000000d").unwrap(),
            env_id: uuid::Uuid::parse_str("00000000-0000-0000-0000-00000000000e").unwrap(),
            status: "ACTIVE".into(),
            scopes: vec!["accounts:read".into(), "transactions:read".into()],
            created_at: chrono::DateTime::parse_from_rfc3339("2026-03-10T12:00:00Z")
                .unwrap()
                .into(),
        }
    }

    #[test]
    fn consent_serde_roundtrip() {
        let c = sample_consent();
        let json = serde_json::to_string(&c).unwrap();
        assert!(json.contains("\"status\":\"ACTIVE\""));
        assert!(json.contains("\"scopes\":[\"accounts:read\",\"transactions:read\"]"));
        let de: Consent = serde_json::from_str(&json).unwrap();
        assert_eq!(de.id, c.id);
        assert_eq!(de.env_id, c.env_id);
        assert_eq!(de.status, c.status);
        assert_eq!(de.scopes, c.scopes);
        assert_eq!(de.created_at, c.created_at);
    }

    #[test]
    fn consent_deserializes_from_json_object() {
        let json = serde_json::json!({
            "id": "00000000-0000-0000-0000-000000000001",
            "env_id": "00000000-0000-0000-0000-000000000002",
            "status": "REVOKED",
            "scopes": [],
            "created_at": "2026-04-01T00:00:00Z"
        });
        let c: Consent = serde_json::from_value(json).unwrap();
        assert_eq!(c.status, "REVOKED");
        assert!(c.scopes.is_empty());
    }
}
