// SPDX-FileCopyrightText: Copyright Logically Magic Inc. <hello@logicallymagic.com>
// SPDX-FileCopyrightText: Copyright Hamid Jahanzad <Hamid@Jahanzad.com>
// SPDX-FileCopyrightText: Copyright Contributors to the Bank8:SDK
// SPDX-License-Identifier: Apache-2.0

//! # Payments client
//!
//! Rust client for the Bank8:Payments payment-orchestration service -
//! instructions, payments, payment instruments, account links, rules,
//! and files. See `services/payments/DESIGN.md`.
//!
//! # Examples
//!
//! ```no_run
//! # use bank8::payments::{PaymentsClient, PaymentsError};
//! # async fn demo() -> Result<(), PaymentsError> {
//! let client = PaymentsClient::new("https://bank8-payments.ops.sys.local.sentin8.dev").build();
//! let instructions = client.list_instructions(None).await?;
//! # Ok(())
//! # }
//! ```

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{prepare_request, ClientBuilder};

/// Payments client errors.
#[derive(Debug, thiserror::Error)]
pub enum PaymentsError {
    /// HTTP transport error.
    #[error("payments http error: {0}")]
    Http(String),
    /// Response decode error.
    #[error("payments decode error: {0}")]
    Decode(String),
    /// Non-success status code.
    #[error("payments status {status}: {body}")]
    Status {
        /// HTTP status code.
        status: u16,
        /// Response body text.
        body: String,
    },
}

/// Payments client.
#[derive(Clone)]
pub struct PaymentsClient {
    base_url: String,
    auth_token: Option<String>,
    http_client: reqwest::Client,
}

impl PaymentsClient {
    /// Create a builder seeded with `base_url`.
    // Deliberate builder pattern (doc example: `PaymentsClient::new(url).build `).
    #[allow(clippy::new_ret_no_self)]
    pub fn new(base_url: impl Into<String>) -> PaymentsClientBuilder {
        PaymentsClientBuilder {
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
    ) -> Result<T, PaymentsError> {
        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            let body = resp.text().await.unwrap_or_default();
            return Err(PaymentsError::Status { status, body });
        }
        resp.json()
            .await
            .map_err(|e| PaymentsError::Decode(e.to_string()))
    }

    async fn decode_empty(resp: reqwest::Response) -> Result<(), PaymentsError> {
        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            let body = resp.text().await.unwrap_or_default();
            return Err(PaymentsError::Status { status, body });
        }
        Ok(())
    }

    // ── Instructions ──────────────────────────────────────────────────

    /// `GET /v1/instructions` - list instructions (optional state filter).
    pub async fn list_instructions(
        &self,
        state: Option<&str>,
    ) -> Result<Vec<Instruction>, PaymentsError> {
        let path = match state {
            Some(s) => format!("/v1/instructions?state={s}"),
            None => "/v1/instructions".to_string(),
        };
        let resp = prepare_request(
            &self.http_client,
            reqwest::Method::GET,
            &self.url(&path),
            self.auth_token.as_deref(),
        )
        .send()
        .await
        .map_err(|e| PaymentsError::Http(e.to_string()))?;
        Self::decode(resp).await
    }

    /// `GET /v1/instructions/{id}` - get an instruction.
    pub async fn get_instruction(&self, id: Uuid) -> Result<Instruction, PaymentsError> {
        let resp = prepare_request(
            &self.http_client,
            reqwest::Method::GET,
            &self.url(&format!("/v1/instructions/{id}")),
            self.auth_token.as_deref(),
        )
        .send()
        .await
        .map_err(|e| PaymentsError::Http(e.to_string()))?;
        Self::decode(resp).await
    }

    /// `POST /v1/instructions` - create an instruction.
    pub async fn create_instruction(
        &self,
        req: &CreateInstructionRequest,
    ) -> Result<Instruction, PaymentsError> {
        let resp = prepare_request(
            &self.http_client,
            reqwest::Method::POST,
            &self.url("/v1/instructions"),
            self.auth_token.as_deref(),
        )
        .json(req)
        .send()
        .await
        .map_err(|e| PaymentsError::Http(e.to_string()))?;
        Self::decode(resp).await
    }

    /// `POST /v1/instructions/{id}/retry` - retry an instruction.
    pub async fn retry_instruction(&self, id: Uuid) -> Result<Instruction, PaymentsError> {
        let resp = prepare_request(
            &self.http_client,
            reqwest::Method::POST,
            &self.url(&format!("/v1/instructions/{id}/retry")),
            self.auth_token.as_deref(),
        )
        .send()
        .await
        .map_err(|e| PaymentsError::Http(e.to_string()))?;
        Self::decode(resp).await
    }

    /// `POST /v1/instructions/{id}/cancel` - cancel an instruction.
    pub async fn cancel_instruction(&self, id: Uuid) -> Result<(), PaymentsError> {
        let resp = prepare_request(
            &self.http_client,
            reqwest::Method::POST,
            &self.url(&format!("/v1/instructions/{id}/cancel")),
            self.auth_token.as_deref(),
        )
        .send()
        .await
        .map_err(|e| PaymentsError::Http(e.to_string()))?;
        Self::decode_empty(resp).await
    }

    /// `POST /v1/instructions/{id}/repair` - repair an instruction with new input.
    pub async fn repair_instruction(
        &self,
        id: Uuid,
        req: &RepairInstructionRequest,
    ) -> Result<Instruction, PaymentsError> {
        let resp = prepare_request(
            &self.http_client,
            reqwest::Method::POST,
            &self.url(&format!("/v1/instructions/{id}/repair")),
            self.auth_token.as_deref(),
        )
        .json(req)
        .send()
        .await
        .map_err(|e| PaymentsError::Http(e.to_string()))?;
        Self::decode(resp).await
    }

    // ── Payments ──────────────────────────────────────────────────────

    /// `GET /v1/payments` - list payments.
    pub async fn list_payments(&self) -> Result<Vec<Payment>, PaymentsError> {
        let resp = prepare_request(
            &self.http_client,
            reqwest::Method::GET,
            &self.url("/v1/payments"),
            self.auth_token.as_deref(),
        )
        .send()
        .await
        .map_err(|e| PaymentsError::Http(e.to_string()))?;
        Self::decode(resp).await
    }

    /// `GET /v1/payments/{id}` - get a payment.
    pub async fn get_payment(&self, id: Uuid) -> Result<Payment, PaymentsError> {
        let resp = prepare_request(
            &self.http_client,
            reqwest::Method::GET,
            &self.url(&format!("/v1/payments/{id}")),
            self.auth_token.as_deref(),
        )
        .send()
        .await
        .map_err(|e| PaymentsError::Http(e.to_string()))?;
        Self::decode(resp).await
    }

    // ── Payment Instruments ───────────────────────────────────────────

    /// `GET /v1/payment-instruments` - list payment instruments.
    pub async fn list_payment_instruments(&self) -> Result<Vec<PaymentInstrument>, PaymentsError> {
        let resp = prepare_request(
            &self.http_client,
            reqwest::Method::GET,
            &self.url("/v1/payment-instruments"),
            self.auth_token.as_deref(),
        )
        .send()
        .await
        .map_err(|e| PaymentsError::Http(e.to_string()))?;
        Self::decode(resp).await
    }

    /// `POST /v1/payment-instruments` - create a payment instrument.
    pub async fn create_payment_instrument(
        &self,
        req: &CreatePaymentInstrumentRequest,
    ) -> Result<PaymentInstrument, PaymentsError> {
        let resp = prepare_request(
            &self.http_client,
            reqwest::Method::POST,
            &self.url("/v1/payment-instruments"),
            self.auth_token.as_deref(),
        )
        .json(req)
        .send()
        .await
        .map_err(|e| PaymentsError::Http(e.to_string()))?;
        Self::decode(resp).await
    }

    // ── Account Links ──────────────────────────────────────────────────

    /// `GET /v1/account-links` - list account links.
    pub async fn list_account_links(&self) -> Result<Vec<AccountLink>, PaymentsError> {
        let resp = prepare_request(
            &self.http_client,
            reqwest::Method::GET,
            &self.url("/v1/account-links"),
            self.auth_token.as_deref(),
        )
        .send()
        .await
        .map_err(|e| PaymentsError::Http(e.to_string()))?;
        Self::decode(resp).await
    }

    /// `POST /v1/account-links` - create an account link.
    pub async fn create_account_link(
        &self,
        req: &CreateAccountLinkRequest,
    ) -> Result<AccountLink, PaymentsError> {
        let resp = prepare_request(
            &self.http_client,
            reqwest::Method::POST,
            &self.url("/v1/account-links"),
            self.auth_token.as_deref(),
        )
        .json(req)
        .send()
        .await
        .map_err(|e| PaymentsError::Http(e.to_string()))?;
        Self::decode(resp).await
    }

    // ── Rules ──────────────────────────────────────────────────────────

    /// `GET /v1/rules` - list rules.
    pub async fn list_rules(&self) -> Result<Vec<Rule>, PaymentsError> {
        let resp = prepare_request(
            &self.http_client,
            reqwest::Method::GET,
            &self.url("/v1/rules"),
            self.auth_token.as_deref(),
        )
        .send()
        .await
        .map_err(|e| PaymentsError::Http(e.to_string()))?;
        Self::decode(resp).await
    }

    /// `POST /v1/rules` - create a rule.
    pub async fn create_rule(&self, req: &CreateRuleRequest) -> Result<Rule, PaymentsError> {
        let resp = prepare_request(
            &self.http_client,
            reqwest::Method::POST,
            &self.url("/v1/rules"),
            self.auth_token.as_deref(),
        )
        .json(req)
        .send()
        .await
        .map_err(|e| PaymentsError::Http(e.to_string()))?;
        Self::decode(resp).await
    }

    // ── Files ──────────────────────────────────────────────────────────

    /// `GET /v1/files` - list files.
    pub async fn list_files(&self) -> Result<Vec<FileSummary>, PaymentsError> {
        let resp = prepare_request(
            &self.http_client,
            reqwest::Method::GET,
            &self.url("/v1/files"),
            self.auth_token.as_deref(),
        )
        .send()
        .await
        .map_err(|e| PaymentsError::Http(e.to_string()))?;
        Self::decode(resp).await
    }

    /// `POST /v1/files` - upload a file.
    pub async fn upload_file(&self, req: &UploadFileRequest) -> Result<FileSummary, PaymentsError> {
        let resp = prepare_request(
            &self.http_client,
            reqwest::Method::POST,
            &self.url("/v1/files"),
            self.auth_token.as_deref(),
        )
        .json(req)
        .send()
        .await
        .map_err(|e| PaymentsError::Http(e.to_string()))?;
        Self::decode(resp).await
    }
}

// ── Builder ───────────────────────────────────────────────────────────

/// Builder for [`PaymentsClient`].
#[derive(Clone)]
pub struct PaymentsClientBuilder {
    base_url: String,
    auth_token: Option<String>,
    http_client: reqwest::Client,
}

impl ClientBuilder for PaymentsClientBuilder {
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

impl PaymentsClientBuilder {
    /// Finalize the builder into a [`PaymentsClient`].
    pub fn build(self) -> PaymentsClient {
        PaymentsClient {
            base_url: self.base_url.trim_end_matches('/').to_string(),
            auth_token: self.auth_token,
            http_client: self.http_client,
        }
    }
}

// ── Request / Response types ────────────────────────────────────────────

/// A payment instruction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Instruction {
    /// Instruction ID.
    pub id: Uuid,
    /// Organization ID.
    pub org_id: Uuid,
    /// Environment ID.
    pub env_id: Uuid,
    /// Logic ID.
    pub instruction_logic_id: String,
    /// State.
    pub state: String,
    /// Correlation ID.
    pub correlation_id: String,
    /// Input payload.
    pub input: serde_json::Value,
    /// Output payload (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<serde_json::Value>,
    /// Creation timestamp.
    pub created_at: DateTime<Utc>,
}

/// Request body for `create_instruction`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateInstructionRequest {
    /// Logic ID.
    pub instruction_logic_id: String,
    /// Input payload.
    pub input: serde_json::Value,
}

/// Request body for `repair_instruction`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepairInstructionRequest {
    /// New input payload.
    pub input: serde_json::Value,
}

/// A payment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Payment {
    /// Payment ID.
    pub id: Uuid,
    /// Organization ID.
    pub org_id: Uuid,
    /// Environment ID.
    pub env_id: Uuid,
    /// Instruction ID.
    pub instruction_id: Uuid,
    /// State.
    pub state: String,
    /// Correlation ID.
    pub correlation_id: String,
    /// Creation timestamp.
    pub created_at: DateTime<Utc>,
}

/// A payment instrument.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentInstrument {
    /// Instrument ID.
    pub id: Uuid,
    /// Organization ID.
    pub org_id: Uuid,
    /// Instrument type.
    pub instrument_type: String,
    /// Status.
    pub status: String,
    /// Creation timestamp.
    pub created_at: DateTime<Utc>,
}

/// Request body for `create_payment_instrument`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePaymentInstrumentRequest {
    /// Instrument type.
    pub instrument_type: String,
    /// Instrument payload.
    pub payload: serde_json::Value,
}

/// An account link.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountLink {
    /// Link ID.
    pub id: Uuid,
    /// Organization ID.
    pub org_id: Uuid,
    /// Account ID (Ledger).
    pub account_id: Uuid,
    /// Integration ID.
    pub integration_id: Uuid,
    /// Status.
    pub status: String,
    /// Creation timestamp.
    pub created_at: DateTime<Utc>,
}

/// Request body for `create_account_link`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAccountLinkRequest {
    /// Account ID (Ledger).
    pub account_id: Uuid,
    /// Integration ID.
    pub integration_id: Uuid,
}

/// A rule.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    /// Rule ID.
    pub id: Uuid,
    /// Organization ID.
    pub org_id: Uuid,
    /// Name.
    pub name: String,
    /// Status.
    pub status: String,
    /// Creation timestamp.
    pub created_at: DateTime<Utc>,
}

/// Request body for `create_rule`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateRuleRequest {
    /// Name.
    pub name: String,
    /// Rule definition.
    pub definition: serde_json::Value,
}

/// A file summary.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileSummary {
    /// File ID.
    pub id: Uuid,
    /// Organization ID.
    pub org_id: Uuid,
    /// Filename.
    pub filename: String,
    /// Content type.
    pub content_type: String,
    /// Size in bytes.
    pub size: i64,
    /// Creation timestamp.
    pub created_at: DateTime<Utc>,
}

/// Request body for `upload_file`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadFileRequest {
    /// Filename.
    pub filename: String,
    /// Content type.
    pub content_type: String,
    /// Base64-encoded content.
    pub content: String,
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
        let client = PaymentsClient::new("https://bank8-payments.ops.sys.local.sentin8.dev/")
            .with_token("tok")
            .build();
        assert_eq!(
            client.base_url,
            "https://bank8-payments.ops.sys.local.sentin8.dev"
        );
        assert_eq!(client.auth_token.as_deref(), Some("tok"));
    }

    #[test]
    fn create_instruction_request_serializes() {
        provider();
        let req = CreateInstructionRequest {
            instruction_logic_id: "a2a-credit-transfer".into(),
            input: serde_json::json!({"amount": "10.00", "currency": "USD"}),
        };
        let json = serde_json::to_value(&req).unwrap();
        assert_eq!(json["instruction_logic_id"], "a2a-credit-transfer");
        assert_eq!(json["input"]["amount"], "10.00");
    }

    #[test]
    fn instruction_deserializes() {
        provider();
        let json = r#"{
            "id":"00000000-0000-0000-0000-000000000001",
            "org_id":"00000000-0000-0000-0000-000000000002",
            "env_id":"00000000-0000-0000-0000-000000000003",
            "instruction_logic_id":"a2a",
            "state":"pending",
            "correlation_id":"c-1",
            "input":{"amount":"10.00"},
            "created_at":"2026-01-01T00:00:00Z"
        }"#;
        let i: Instruction = serde_json::from_str(json).unwrap();
        assert_eq!(i.state, "pending");
        assert_eq!(i.instruction_logic_id, "a2a");
    }

    #[test]
    fn payment_deserializes() {
        provider();
        let json = r#"{
            "id":"00000000-0000-0000-0000-000000000001",
            "org_id":"00000000-0000-0000-0000-000000000002",
            "env_id":"00000000-0000-0000-0000-000000000003",
            "instruction_id":"00000000-0000-0000-0000-000000000004",
            "state":"completed",
            "correlation_id":"c-1",
            "created_at":"2026-01-01T00:00:00Z"
        }"#;
        let p: Payment = serde_json::from_str(json).unwrap();
        assert_eq!(p.state, "completed");
    }

    #[tokio::test]
    async fn retry_instruction_handles_error_status() {
        let mut server = mockito::Server::new_async().await;
        let id = Uuid::parse_str("00000000-0000-0000-0000-000000000099").unwrap();
        let _m = server
            .mock(
                "POST",
                "/v1/instructions/00000000-0000-0000-0000-000000000099/retry",
            )
            .with_status(409)
            .with_body("conflict")
            .create_async()
            .await;
        let client = PaymentsClient::new(server.url()).build();
        let err = client.retry_instruction(id).await.unwrap_err();
        match err {
            PaymentsError::Status { status, body } => {
                assert_eq!(status, 409);
                assert_eq!(body, "conflict");
            }
            other => panic!("expected Status, got {other:?}"),
        }
    }
}
