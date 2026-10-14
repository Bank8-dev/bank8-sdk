// SPDX-FileCopyrightText: Copyright Logically Magic Inc. <hello@logicallymagic.com>
// SPDX-FileCopyrightText: Copyright Hamid Jahanzad <Hamid@Jahanzad.com>
// SPDX-FileCopyrightText: Copyright Contributors to the Bank8:SDK
// SPDX-License-Identifier: Apache-2.0

//! # bank8-ledger-contracts
//!
//! Wire types for the Bank8:Ledger HTTP API. See the crate `README.md` for
//! the design overview and the wire-format invariants.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A ledger account as returned by the account reads.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    /// Account id (globally unique).
    pub id: Uuid,
    /// Owning tenant environment.
    pub env_id: Uuid,
    /// Product the account is typed by.
    pub product_id: Uuid,
    /// Display name.
    pub name: String,
    /// Lifecycle state (`OPEN`, `PENDING`, `CLOSED`).
    pub status: String,
    /// ISO 4217 currency code.
    pub denomination: String,
    /// Creation timestamp.
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// A point-in-time balance reading for one account.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Balance {
    /// Account the balance belongs to.
    pub account_id: Uuid,
    /// Balance phase (`AVAILABLE`, `COMMITTED`, …).
    pub balance_type: String,
    /// Amount as a JSON string on the wire (money is never a float).
    pub amount: String,
    /// Currency code of the amount.
    pub currency: String,
    /// As-of timestamp for the reading.
    pub as_of: chrono::DateTime<chrono::Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_account() -> Account {
        Account {
            id: uuid::Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap(),
            env_id: uuid::Uuid::parse_str("00000000-0000-0000-0000-000000000002").unwrap(),
            product_id: uuid::Uuid::parse_str("00000000-0000-0000-0000-000000000003").unwrap(),
            name: "Everyday Chequing".into(),
            status: "OPEN".into(),
            denomination: "CAD".into(),
            created_at: chrono::DateTime::parse_from_rfc3339("2026-01-15T10:30:00Z")
                .unwrap()
                .into(),
        }
    }

    fn sample_balance() -> Balance {
        Balance {
            account_id: uuid::Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap(),
            balance_type: "AVAILABLE".into(),
            amount: "1234.56".into(),
            currency: "CAD".into(),
            as_of: chrono::DateTime::parse_from_rfc3339("2026-09-04T08:00:00Z")
                .unwrap()
                .into(),
        }
    }

    #[test]
    fn account_serde_roundtrip() {
        let a = sample_account();
        let json = serde_json::to_string(&a).unwrap();
        assert!(json.contains("\"denomination\":\"CAD\""));
        assert!(json.contains("\"status\":\"OPEN\""));
        let de: Account = serde_json::from_str(&json).unwrap();
        assert_eq!(de.id, a.id);
        assert_eq!(de.env_id, a.env_id);
        assert_eq!(de.product_id, a.product_id);
        assert_eq!(de.name, a.name);
        assert_eq!(de.status, a.status);
        assert_eq!(de.denomination, a.denomination);
        assert_eq!(de.created_at, a.created_at);
    }

    #[test]
    fn balance_serde_roundtrip() {
        let b = sample_balance();
        let json = serde_json::to_string(&b).unwrap();
        // Amount stays a JSON string (wire format for money fields).
        assert!(json.contains("\"amount\":\"1234.56\""));
        let de: Balance = serde_json::from_str(&json).unwrap();
        assert_eq!(de.account_id, b.account_id);
        assert_eq!(de.balance_type, b.balance_type);
        assert_eq!(de.amount, b.amount);
        assert_eq!(de.currency, b.currency);
        assert_eq!(de.as_of, b.as_of);
    }

    #[test]
    fn account_deserializes_from_json_object() {
        let json = serde_json::json!({
            "id": "00000000-0000-0000-0000-00000000000a",
            "env_id": "00000000-0000-0000-0000-00000000000b",
            "product_id": "00000000-0000-0000-0000-00000000000c",
            "name": "Savings",
            "status": "PENDING",
            "denomination": "USD",
            "created_at": "2026-02-01T00:00:00Z"
        });
        let a: Account = serde_json::from_value(json).unwrap();
        assert_eq!(a.name, "Savings");
        assert_eq!(a.status, "PENDING");
        assert_eq!(a.denomination, "USD");
    }
}