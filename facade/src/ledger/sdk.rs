// SPDX-FileCopyrightText: Copyright Logically Magic Inc. <hello@logicallymagic.com>
// SPDX-FileCopyrightText: Copyright Hamid Jahanzad <Hamid@Jahanzad.com>
// SPDX-FileCopyrightText: Copyright Contributors to the Bank8:SDK
// SPDX-License-Identifier: Apache-2.0

//! # Ledger client
//!
//! Rust client for the Bank8:Ledger core-ledger service - accounts, products,
//! postings, balances, and schedules. See `services/ledger/DESIGN.md`.
//!
//! # Examples
//!
//! ```no_run
//! # use bank8::ledger::{LedgerClient, LedgerError};
//! # async fn demo() -> Result<(), LedgerError> {
//! let client = LedgerClient::new("https://bank8-ledger.ops.sys.local.sentin8.dev").build();
//! let accounts = client.list_accounts(None).await?;
//! # Ok(())
//! # }
//! ```

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{prepare_request, ClientBuilder};

/// Ledger client errors.
#[derive(Debug, thiserror::Error)]
pub enum LedgerError {
    /// HTTP transport error.
    #[error("ledger http error: {0}")]
    Http(String),
    /// Response decode error.
    #[error("ledger decode error: {0}")]
    Decode(String),
    /// Non-success status code.
    #[error("ledger status {status}: {body}")]
    Status {
        /// HTTP status code.
        status: u16,
        /// Response body text.
        body: String,
    },
}

/// Ledger client.
#[derive(Clone)]
pub struct LedgerClient {
    base_url: String,
    auth_token: Option<String>,
    http_client: reqwest::Client,
}

impl LedgerClient {
    /// Create a builder seeded with `base_url`.
    // Deliberate builder pattern (doc example: `LedgerClient::new(url).build `).
    #[allow(clippy::new_ret_no_self)]
    pub fn new(base_url: impl Into<String>) -> LedgerClientBuilder {
        LedgerClientBuilder {
            base_url: base_url.into(),
            auth_token: None,
            http_client: reqwest::Client::new(),
        }
    }

    fn url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }

    async fn decode<T: for<'de> Deserialize<'de>>(
        resp: reqwest::Response,
    ) -> Result<T, LedgerError> {
        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            let body = resp.text().await.unwrap_or_default();
            return Err(LedgerError::Status { status, body });
        }
        resp.json()
            .await
            .map_err(|e| LedgerError::Decode(e.to_string()))
    }

    // ── Accounts ──────────────────────────────────────────────────────

    /// `GET /v1/accounts` - list accounts (optional status filter).
    pub async fn list_accounts(&self, status: Option<&str>) -> Result<Vec<Account>, LedgerError> {
        let path = match status {
            Some(s) => format!("/v1/accounts?status={s}"),
            None => "/v1/accounts".to_string(),
        };
        let resp = prepare_request(
            &self.http_client,
            reqwest::Method::GET,
            &self.url(&path),
            self.auth_token.as_deref(),
        )
        .send()
        .await
        .map_err(|e| LedgerError::Http(e.to_string()))?;
        Self::decode(resp).await
    }

    /// `GET /v1/accounts/{id}` - get an account.
    pub async fn get_account(&self, id: Uuid) -> Result<Account, LedgerError> {
        let resp = prepare_request(
            &self.http_client,
            reqwest::Method::GET,
            &self.url(&format!("/v1/accounts/{id}")),
            self.auth_token.as_deref(),
        )
        .send()
        .await
        .map_err(|e| LedgerError::Http(e.to_string()))?;
        Self::decode(resp).await
    }

    /// `POST /v1/accounts` - create an account.
    pub async fn create_account(&self, req: &CreateAccountRequest) -> Result<Account, LedgerError> {
        let resp = prepare_request(
            &self.http_client,
            reqwest::Method::POST,
            &self.url("/v1/accounts"),
            self.auth_token.as_deref(),
        )
        .json(req)
        .send()
        .await
        .map_err(|e| LedgerError::Http(e.to_string()))?;
        Self::decode(resp).await
    }

    // ── Products ──────────────────────────────────────────────────────

    /// `GET /v1/products` - list products.
    pub async fn list_products(&self) -> Result<Vec<Product>, LedgerError> {
        let resp = prepare_request(
            &self.http_client,
            reqwest::Method::GET,
            &self.url("/v1/products"),
            self.auth_token.as_deref(),
        )
        .send()
        .await
        .map_err(|e| LedgerError::Http(e.to_string()))?;
        Self::decode(resp).await
    }

    /// `POST /v1/products` - create a product.
    pub async fn create_product(&self, req: &CreateProductRequest) -> Result<Product, LedgerError> {
        let resp = prepare_request(
            &self.http_client,
            reqwest::Method::POST,
            &self.url("/v1/products"),
            self.auth_token.as_deref(),
        )
        .json(req)
        .send()
        .await
        .map_err(|e| LedgerError::Http(e.to_string()))?;
        Self::decode(resp).await
    }

    // ── Postings ──────────────────────────────────────────────────────

    /// `POST /v1/postings` - create a posting batch.
    pub async fn create_postings(
        &self,
        req: &CreatePostingsRequest,
    ) -> Result<PostingBatch, LedgerError> {
        let resp = prepare_request(
            &self.http_client,
            reqwest::Method::POST,
            &self.url("/v1/postings"),
            self.auth_token.as_deref(),
        )
        .json(req)
        .send()
        .await
        .map_err(|e| LedgerError::Http(e.to_string()))?;
        Self::decode(resp).await
    }

    // ── Balances ──────────────────────────────────────────────────────

    /// `GET /v1/accounts/{id}/balances` - get balances for an account.
    pub async fn get_balances(&self, account_id: Uuid) -> Result<Vec<Balance>, LedgerError> {
        let resp = prepare_request(
            &self.http_client,
            reqwest::Method::GET,
            &self.url(&format!("/v1/accounts/{account_id}/balances")),
            self.auth_token.as_deref(),
        )
        .send()
        .await
        .map_err(|e| LedgerError::Http(e.to_string()))?;
        Self::decode(resp).await
    }

    // ── Schedules ─────────────────────────────────────────────────────

    /// `GET /v1/schedules` - list schedules.
    pub async fn list_schedules(&self) -> Result<Vec<Schedule>, LedgerError> {
        let resp = prepare_request(
            &self.http_client,
            reqwest::Method::GET,
            &self.url("/v1/schedules"),
            self.auth_token.as_deref(),
        )
        .send()
        .await
        .map_err(|e| LedgerError::Http(e.to_string()))?;
        Self::decode(resp).await
    }

    /// `POST /v1/schedules` - create a schedule.
    pub async fn create_schedule(
        &self,
        req: &CreateScheduleRequest,
    ) -> Result<Schedule, LedgerError> {
        let resp = prepare_request(
            &self.http_client,
            reqwest::Method::POST,
            &self.url("/v1/schedules"),
            self.auth_token.as_deref(),
        )
        .json(req)
        .send()
        .await
        .map_err(|e| LedgerError::Http(e.to_string()))?;
        Self::decode(resp).await
    }
}

// ── Builder ───────────────────────────────────────────────────────────

/// Builder for [`LedgerClient`].
#[derive(Clone)]
pub struct LedgerClientBuilder {
    base_url: String,
    auth_token: Option<String>,
    http_client: reqwest::Client,
}

impl ClientBuilder for LedgerClientBuilder {
    fn with_base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = url.into().trim_end_matches('/').to_string();
        self
    }
    fn with_token(mut self, token: impl Into<String>) -> Self {
        self.auth_token = Some(token.into());
        self
    }
    fn with_http_client(mut self, client: reqwest::Client) -> Self {
        self.http_client = client;
        self
    }
}

impl LedgerClientBuilder {
    /// Finalize the builder into a [`LedgerClient`].
    pub fn build(self) -> LedgerClient {
        LedgerClient {
            base_url: self.base_url.trim_end_matches('/').to_string(),
            auth_token: self.auth_token,
            http_client: self.http_client,
        }
    }
}

// ── Request / Response types ────────────────────────────────────────────

/// A ledger account.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    /// Account ID.
    pub id: Uuid,
    /// Organization ID.
    pub org_id: Uuid,
    /// Environment ID.
    pub env_id: Uuid,
    /// Product ID.
    pub product_id: Uuid,
    /// Account name.
    pub name: String,
    /// Status.
    pub status: String,
    /// Denomination (currency).
    pub denomination: String,
    /// Creation timestamp.
    pub created_at: DateTime<Utc>,
}

/// Request body for `create_account`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAccountRequest {
    /// Product ID.
    pub product_id: Uuid,
    /// Account name.
    pub name: String,
    /// Denomination (currency).
    pub denomination: String,
    /// Optional parameters.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<serde_json::Value>,
}

/// A product definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Product {
    /// Product ID.
    pub id: Uuid,
    /// Product type.
    pub product_type: String,
    /// Name.
    pub name: String,
    /// Version.
    pub version: String,
    /// Status.
    pub status: String,
    /// Parameters.
    #[serde(default)]
    pub parameters: serde_json::Value,
    /// Modules.
    #[serde(default)]
    pub modules: serde_json::Value,
}

/// Request body for `create_product`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateProductRequest {
    /// Product type.
    pub product_type: String,
    /// Name.
    pub name: String,
    /// Parameters.
    #[serde(default)]
    pub parameters: serde_json::Value,
    /// Modules.
    #[serde(default)]
    pub modules: serde_json::Value,
}

/// Request body for `create_postings`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePostingsRequest {
    /// Posting instructions.
    pub instructions: Vec<PostingInstruction>,
    /// Whether to run asynchronously.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub async_run: Option<bool>,
}

/// A single posting instruction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostingInstruction {
    /// Account ID.
    pub account_id: Uuid,
    /// Amount (string to preserve precision).
    pub amount: String,
    /// Denomination.
    pub denomination: String,
    /// Direction: `debit` or `credit`.
    pub direction: String,
    /// Description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Result of a posting batch.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostingBatch {
    /// Batch ID.
    pub id: Uuid,
    /// Status.
    pub status: String,
    /// Instruction count.
    pub instruction_count: i32,
    /// Creation timestamp.
    pub created_at: DateTime<Utc>,
    /// Completion timestamp (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<DateTime<Utc>>,
}

/// A balance entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Balance {
    /// Account ID.
    pub account_id: Uuid,
    /// Balance type.
    pub balance_type: String,
    /// Amount (string to preserve precision).
    pub amount: String,
    /// Currency.
    pub currency: String,
    /// As-of timestamp.
    pub as_of: DateTime<Utc>,
}

/// A schedule.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Schedule {
    /// Schedule ID.
    pub id: Uuid,
    /// Name.
    pub name: String,
    /// Description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Timezone.
    pub timezone: String,
    /// Whether active.
    pub is_active: bool,
    /// Creation timestamp.
    pub created_at: DateTime<Utc>,
}

/// Request body for `create_schedule`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateScheduleRequest {
    /// Name.
    pub name: String,
    /// Description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Timezone.
    pub timezone: String,
    /// Cron expression.
    pub cron: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::install_test_provider;

    fn provider() {
        install_test_provider();
    }

    #[test]
    fn builder_attaches_token_and_strips_slash() {
        provider();
        let client = LedgerClient::new("https://bank8-ledger.ops.sys.local.sentin8.dev/")
            .with_token("tok")
            .build();
        assert_eq!(
            client.base_url,
            "https://bank8-ledger.ops.sys.local.sentin8.dev"
        );
        assert_eq!(client.auth_token.as_deref(), Some("tok"));
    }

    #[test]
    fn create_postings_request_serializes() {
        provider();
        let req = CreatePostingsRequest {
            instructions: vec![PostingInstruction {
                account_id: Uuid::nil(),
                amount: "100.00".into(),
                denomination: "USD".into(),
                direction: "credit".into(),
                description: Some("test".into()),
            }],
            async_run: None,
        };
        let json = serde_json::to_value(&req).unwrap();
        assert_eq!(json["instructions"][0]["amount"], "100.00");
        assert_eq!(json["instructions"][0]["direction"], "credit");
    }

    #[test]
    fn account_deserializes() {
        provider();
        let json = r#"{
            "id":"00000000-0000-0000-0000-000000000001",
            "org_id":"00000000-0000-0000-0000-000000000002",
            "env_id":"00000000-0000-0000-0000-000000000003",
            "product_id":"00000000-0000-0000-0000-000000000004",
            "name":"Checking",
            "status":"active",
            "denomination":"USD",
            "created_at":"2026-01-01T00:00:00Z"
        }"#;
        let a: Account = serde_json::from_str(json).unwrap();
        assert_eq!(a.name, "Checking");
        assert_eq!(a.denomination, "USD");
    }

    #[test]
    fn balance_deserializes() {
        provider();
        let json = r#"{
            "account_id":"00000000-0000-0000-0000-000000000001",
            "balance_type":"available",
            "amount":"100.00",
            "currency":"USD",
            "as_of":"2026-01-01T00:00:00Z"
        }"#;
        let b: Balance = serde_json::from_str(json).unwrap();
        assert_eq!(b.balance_type, "available");
        assert_eq!(b.amount, "100.00");
    }

    #[tokio::test]
    async fn get_account_handles_error_status() {
        let mut server = mockito::Server::new_async().await;
        let id = Uuid::parse_str("00000000-0000-0000-0000-000000000099").unwrap();
        let _m = server
            .mock("GET", "/v1/accounts/00000000-0000-0000-0000-000000000099")
            .with_status(404)
            .with_body("not found")
            .create_async()
            .await;
        let client = LedgerClient::new(server.url()).build();
        let err = client.get_account(id).await.unwrap_err();
        match err {
            LedgerError::Status { status, body } => {
                assert_eq!(status, 404);
                assert_eq!(body, "not found");
            }
            other => panic!("expected Status, got {other:?}"),
        }
    }
}
