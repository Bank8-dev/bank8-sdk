// SPDX-FileCopyrightText: Copyright Logically Magic Inc. <hello@logicallymagic.com>
// SPDX-FileCopyrightText: Copyright Hamid Jahanzad <Hamid@Jahanzad.com>
// SPDX-FileCopyrightText: Copyright Contributors to the Bank8:SDK
// SPDX-License-Identifier: Apache-2.0

//! # FDX client
//!
//! Rust client for the Bank8:FDX open-banking service — consents, recipient
//! accounts, transactions, statements, payment networks, customers,
//! transfers, and payments. See `services/fdx/DESIGN.md`.
//!
//! # Examples
//!
//! ```no_run
//! # use bank8::fdx::{Fdx8Client, Fdx8Error};
//! # async fn demo() -> Result<(), Fdx8Error> {
//! let client = Fdx8Client::new("https://bank8-fdx.ops.sys.sentin8.local").build();
//! let consents = client.list_consents().await?;
//! # Ok(())
//! # }
//! ```

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{prepare_request, ClientBuilder};

/// FDX client errors.
#[derive(Debug, thiserror::Error)]
pub enum Fdx8Error {
    /// HTTP transport error.
    #[error("fdx http error: {0}")]
    Http(String),
    /// Response decode error.
    #[error("fdx decode error: {0}")]
    Decode(String),
    /// Non-success status code.
    #[error("fdx status {status}: {body}")]
    Status {
        /// HTTP status code.
        status: u16,
        /// Response body text.
        body: String,
    },
}

/// FDX client.
#[derive(Clone)]
pub struct Fdx8Client {
    base_url: String,
    auth_token: Option<String>,
    http_client: reqwest::Client,
}

impl Fdx8Client {
    /// Create a builder seeded with `base_url`.
    // Deliberate builder pattern (doc example: `Fdx8Client::new(url).build `).
    #[allow(clippy::new_ret_no_self)]
    pub fn new(base_url: impl Into<String>) -> Fdx8ClientBuilder {
        Fdx8ClientBuilder {
            base_url: base_url.into(),
            auth_token: None,
            http_client: reqwest::Client::new(),
        }
    }

    fn url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }

    async fn decode<T: for<'de> Deserialize<'de>>(resp: reqwest::Response) -> Result<T, Fdx8Error> {
        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            let body = resp.text().await.unwrap_or_default();
            return Err(Fdx8Error::Status { status, body });
        }
        resp.json()
            .await
            .map_err(|e| Fdx8Error::Decode(e.to_string()))
    }

    async fn decode_empty(resp: reqwest::Response) -> Result<(), Fdx8Error> {
        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            let body = resp.text().await.unwrap_or_default();
            return Err(Fdx8Error::Status { status, body });
        }
        Ok(())
    }

    // ── Consents ──────────────────────────────────────────────────────

    /// `POST /v1/recipient/consents/start` — start a consent flow.
    pub async fn start_consent(&self, req: &StartConsentRequest) -> Result<Consent, Fdx8Error> {
        let resp = prepare_request(
            &self.http_client,
            reqwest::Method::POST,
            &self.url("/v1/recipient/consents/start"),
            self.auth_token.as_deref(),
        )
        .json(req)
        .send()
        .await
        .map_err(|e| Fdx8Error::Http(e.to_string()))?;
        Self::decode(resp).await
    }

    /// `GET /v1/recipient/consents` — list consents.
    pub async fn list_consents(&self) -> Result<Vec<Consent>, Fdx8Error> {
        let resp = prepare_request(
            &self.http_client,
            reqwest::Method::GET,
            &self.url("/v1/recipient/consents"),
            self.auth_token.as_deref(),
        )
        .send()
        .await
        .map_err(|e| Fdx8Error::Http(e.to_string()))?;
        Self::decode(resp).await
    }

    /// `POST /v1/recipient/consents/{id}/refresh` — refresh a consent.
    pub async fn refresh_consent(&self, id: Uuid) -> Result<Consent, Fdx8Error> {
        let resp = prepare_request(
            &self.http_client,
            reqwest::Method::POST,
            &self.url(&format!("/v1/recipient/consents/{id}/refresh")),
            self.auth_token.as_deref(),
        )
        .send()
        .await
        .map_err(|e| Fdx8Error::Http(e.to_string()))?;
        Self::decode(resp).await
    }

    /// `POST /v1/recipient/consents/{id}/revoke` — revoke a consent.
    pub async fn revoke_consent(&self, id: Uuid) -> Result<(), Fdx8Error> {
        let resp = prepare_request(
            &self.http_client,
            reqwest::Method::POST,
            &self.url(&format!("/v1/recipient/consents/{id}/revoke")),
            self.auth_token.as_deref(),
        )
        .send()
        .await
        .map_err(|e| Fdx8Error::Http(e.to_string()))?;
        Self::decode_empty(resp).await
    }

    // ── Recipient Accounts ────────────────────────────────────────────

    /// `GET /v1/recipient/accounts` — list recipient accounts.
    pub async fn list_recipient_accounts(&self) -> Result<Vec<RecipientAccount>, Fdx8Error> {
        let resp = prepare_request(
            &self.http_client,
            reqwest::Method::GET,
            &self.url("/v1/recipient/accounts"),
            self.auth_token.as_deref(),
        )
        .send()
        .await
        .map_err(|e| Fdx8Error::Http(e.to_string()))?;
        Self::decode(resp).await
    }

    /// `GET /v1/recipient/accounts/{id}/transactions` — get account transactions.
    pub async fn get_account_transactions(&self, id: Uuid) -> Result<Vec<Transaction>, Fdx8Error> {
        let resp = prepare_request(
            &self.http_client,
            reqwest::Method::GET,
            &self.url(&format!("/v1/recipient/accounts/{id}/transactions")),
            self.auth_token.as_deref(),
        )
        .send()
        .await
        .map_err(|e| Fdx8Error::Http(e.to_string()))?;
        Self::decode(resp).await
    }

    /// `GET /fdx/v6/accounts/{id}/statements` — list account statements.
    pub async fn list_account_statements(
        &self,
        account_id: Uuid,
    ) -> Result<Vec<Statement>, Fdx8Error> {
        let resp = prepare_request(
            &self.http_client,
            reqwest::Method::GET,
            &self.url(&format!("/fdx/v6/accounts/{account_id}/statements")),
            self.auth_token.as_deref(),
        )
        .send()
        .await
        .map_err(|e| Fdx8Error::Http(e.to_string()))?;
        Self::decode(resp).await
    }

    /// `GET /fdx/v6/accounts/{id}/payment-networks` — get payment networks.
    pub async fn get_payment_networks(
        &self,
        account_id: Uuid,
    ) -> Result<Vec<PaymentNetwork>, Fdx8Error> {
        let resp = prepare_request(
            &self.http_client,
            reqwest::Method::GET,
            &self.url(&format!("/fdx/v6/accounts/{account_id}/payment-networks")),
            self.auth_token.as_deref(),
        )
        .send()
        .await
        .map_err(|e| Fdx8Error::Http(e.to_string()))?;
        Self::decode(resp).await
    }

    // ── Customers ─────────────────────────────────────────────────────

    /// `GET /fdx/v6/customers` — list customers.
    pub async fn list_customers(&self) -> Result<Vec<Customer>, Fdx8Error> {
        let resp = prepare_request(
            &self.http_client,
            reqwest::Method::GET,
            &self.url("/fdx/v6/customers"),
            self.auth_token.as_deref(),
        )
        .send()
        .await
        .map_err(|e| Fdx8Error::Http(e.to_string()))?;
        Self::decode(resp).await
    }

    /// `GET /fdx/v6/customers/current` — get the current customer.
    pub async fn get_current_customer(&self) -> Result<Customer, Fdx8Error> {
        let resp = prepare_request(
            &self.http_client,
            reqwest::Method::GET,
            &self.url("/fdx/v6/customers/current"),
            self.auth_token.as_deref(),
        )
        .send()
        .await
        .map_err(|e| Fdx8Error::Http(e.to_string()))?;
        Self::decode(resp).await
    }

    // ── Transfers & Payments ──────────────────────────────────────────

    /// `POST /fdx/v6/transfers` — create a transfer.
    pub async fn create_transfer(
        &self,
        req: &CreateTransferRequest,
    ) -> Result<Transfer, Fdx8Error> {
        let resp = prepare_request(
            &self.http_client,
            reqwest::Method::POST,
            &self.url("/fdx/v6/transfers"),
            self.auth_token.as_deref(),
        )
        .json(req)
        .send()
        .await
        .map_err(|e| Fdx8Error::Http(e.to_string()))?;
        Self::decode(resp).await
    }

    /// `POST /fdx/v6/payments` — create a payment.
    pub async fn create_payment(&self, req: &CreatePaymentRequest) -> Result<Payment, Fdx8Error> {
        let resp = prepare_request(
            &self.http_client,
            reqwest::Method::POST,
            &self.url("/fdx/v6/payments"),
            self.auth_token.as_deref(),
        )
        .json(req)
        .send()
        .await
        .map_err(|e| Fdx8Error::Http(e.to_string()))?;
        Self::decode(resp).await
    }
}

// ── Builder ───────────────────────────────────────────────────────────

/// Builder for [`Fdx8Client`].
#[derive(Clone)]
pub struct Fdx8ClientBuilder {
    base_url: String,
    auth_token: Option<String>,
    http_client: reqwest::Client,
}

impl ClientBuilder for Fdx8ClientBuilder {
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

impl Fdx8ClientBuilder {
    /// Finalize the builder into an [`Fdx8Client`].
    pub fn build(self) -> Fdx8Client {
        Fdx8Client {
            base_url: self.base_url.trim_end_matches('/').to_string(),
            auth_token: self.auth_token,
            http_client: self.http_client,
        }
    }
}

// ── Request / Response types ────────────────────────────────────────────

/// A consent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Consent {
    /// Consent ID.
    pub id: Uuid,
    /// Organization ID.
    pub org_id: Uuid,
    /// Environment ID.
    pub env_id: Uuid,
    /// Status.
    pub status: String,
    /// Scopes.
    #[serde(default)]
    pub scopes: Vec<String>,
    /// Creation timestamp.
    pub created_at: DateTime<Utc>,
}

/// Request body for `start_consent`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartConsentRequest {
    /// Provider ID.
    pub provider_id: String,
    /// Scopes to request.
    pub scopes: Vec<String>,
}

/// A recipient account.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipientAccount {
    /// Account ID.
    pub id: Uuid,
    /// Account number (masked).
    pub account_number: String,
    /// Display name.
    pub name: String,
    /// Account type.
    pub account_type: String,
    /// Currency.
    pub currency: String,
}

/// A transaction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    /// Transaction ID.
    pub id: Uuid,
    /// Account ID.
    pub account_id: Uuid,
    /// Amount (string to preserve precision).
    pub amount: String,
    /// Currency.
    pub currency: String,
    /// Description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Booking timestamp.
    pub posted_at: DateTime<Utc>,
}

/// A statement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Statement {
    /// Statement ID.
    pub id: Uuid,
    /// Account ID.
    pub account_id: Uuid,
    /// Period start.
    pub period_start: DateTime<Utc>,
    /// Period end.
    pub period_end: DateTime<Utc>,
}

/// A payment network.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentNetwork {
    /// Network name.
    pub name: String,
    /// Network type.
    pub network_type: String,
    /// Whether the network is enabled.
    pub enabled: bool,
}

/// A customer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Customer {
    /// Customer ID.
    pub id: Uuid,
    /// Display name.
    pub name: String,
    /// Email.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
}

/// Request body for `create_transfer`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTransferRequest {
    /// Source account ID.
    pub from_account_id: Uuid,
    /// Destination account ID.
    pub to_account_id: Uuid,
    /// Amount (string).
    pub amount: String,
    /// Currency.
    pub currency: String,
}

/// A transfer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transfer {
    /// Transfer ID.
    pub id: Uuid,
    /// Status.
    pub status: String,
    /// Amount (string).
    pub amount: String,
    /// Currency.
    pub currency: String,
    /// Creation timestamp.
    pub created_at: DateTime<Utc>,
}

/// Request body for `create_payment`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePaymentRequest {
    /// Source account ID.
    pub from_account_id: Uuid,
    /// Destination account ID.
    pub to_account_id: Uuid,
    /// Amount (string).
    pub amount: String,
    /// Currency.
    pub currency: String,
    /// Payment network.
    pub payment_network: String,
}

/// A payment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Payment {
    /// Payment ID.
    pub id: Uuid,
    /// Status.
    pub status: String,
    /// Amount (string).
    pub amount: String,
    /// Currency.
    pub currency: String,
    /// Creation timestamp.
    pub created_at: DateTime<Utc>,
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
        let client = Fdx8Client::new("https://bank8-fdx.ops.sys.sentin8.local/")
            .with_token("tok")
            .build();
        assert_eq!(client.base_url, "https://bank8-fdx.ops.sys.sentin8.local");
        assert_eq!(client.auth_token.as_deref(), Some("tok"));
    }

    #[test]
    fn start_consent_request_serializes() {
        provider();
        let req = StartConsentRequest {
            provider_id: "plaid".into(),
            scopes: vec!["accounts:read".into(), "payments:write".into()],
        };
        let json = serde_json::to_value(&req).unwrap();
        assert_eq!(json["provider_id"], "plaid");
        assert_eq!(json["scopes"][1], "payments:write");
    }

    #[test]
    fn consent_deserializes() {
        provider();
        let json = r#"{
            "id":"00000000-0000-0000-0000-000000000001",
            "org_id":"00000000-0000-0000-0000-000000000002",
            "env_id":"00000000-0000-0000-0000-000000000003",
            "status":"active",
            "scopes":["accounts:read"],
            "created_at":"2026-01-01T00:00:00Z"
        }"#;
        let c: Consent = serde_json::from_str(json).unwrap();
        assert_eq!(c.status, "active");
        assert_eq!(c.scopes, vec!["accounts:read"]);
    }

    #[test]
    fn transaction_deserializes() {
        provider();
        let json = r#"{
            "id":"00000000-0000-0000-0000-000000000001",
            "account_id":"00000000-0000-0000-0000-000000000002",
            "amount":"-50.00",
            "currency":"USD",
            "description":"Coffee",
            "posted_at":"2026-01-01T00:00:00Z"
        }"#;
        let t: Transaction = serde_json::from_str(json).unwrap();
        assert_eq!(t.amount, "-50.00");
        assert_eq!(t.description.as_deref(), Some("Coffee"));
    }

    #[tokio::test]
    async fn revoke_consent_handles_error_status() {
        let mut server = mockito::Server::new_async().await;
        let id = Uuid::parse_str("00000000-0000-0000-0000-000000000099").unwrap();
        let _m = server
            .mock(
                "POST",
                "/v1/recipient/consents/00000000-0000-0000-0000-000000000099/revoke",
            )
            .with_status(403)
            .with_body("forbidden")
            .create_async()
            .await;
        let client = Fdx8Client::new(server.url()).build();
        let err = client.revoke_consent(id).await.unwrap_err();
        match err {
            Fdx8Error::Status { status, body } => {
                assert_eq!(status, 403);
                assert_eq!(body, "forbidden");
            }
            other => panic!("expected Status, got {other:?}"),
        }
    }
}
