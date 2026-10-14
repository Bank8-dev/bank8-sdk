// SPDX-FileCopyrightText: Copyright Logically Magic Inc. <hello@logicallymagic.com>
// SPDX-FileCopyrightText: Copyright Hamid Jahanzad <Hamid@Jahanzad.com>
// SPDX-FileCopyrightText: Copyright Contributors to the Bank8:SDK
// SPDX-License-Identifier: Apache-2.0

//! # bank8-payments-contracts
//!
//! Wire types for the Bank8:Payments HTTP API. See the crate `README.md`
//! for the design overview and the ISO 20022 catalogue invariants.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A payment instruction as returned by the instruction reads.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Instruction {
    /// Instruction id (globally unique).
    pub id: Uuid,
    /// Owning tenant environment.
    pub env_id: Uuid,
    /// Logic the instruction executes.
    pub instruction_logic_id: String,
    /// Lifecycle state (see the payments design §4.6 state machine).
    pub state: String,
    /// Correlation id for scheme messaging.
    pub correlation_id: String,
    /// Scheme payload (typed ISO 20022 envelope as JSON).
    pub input: serde_json::Value,
    /// Execution result — absent while in flight.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<serde_json::Value>,
    /// Creation timestamp.
    pub created_at: chrono::DateTime<chrono::Utc>,
}

// ── ISO 20022 Payload Type Definitions ──────────────────
// All A2A and Card payload types per
// `crates/payments/contracts/README.md` §4.4. These are shared
// with the Flow TypeScript SDK.

/// ISO 20022 message payload types for Account-to-Account (A2A) and Card flows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PayloadType {
    // ── Account-to-Account (A2A) ──
    /// pain.001 — Customer credit transfer initiation
    CreditTransferInitiation,
    /// pacs.002 — FI-to-FI payment status report
    FiToFiPaymentStatusReport,
    /// pacs.004 — Payment return
    PaymentReturn,
    /// pacs.008 — FI-to-FI customer credit transfer
    FiToFiCustomerCreditTransfer,
    /// pacs.028 — Payment status request
    PaymentStatusRequest,
    /// camt.029 — Resolution of investigation
    ResolutionOfInvestigation,
    /// camt.056 — FI-to-FI payment cancellation request
    FiToFiPaymentCancellationRequest,
    /// admi.007 — Receipt acknowledgement
    ReceiptAcknowledgement,

    // ── Cards ──
    /// cain.001 — Authorisation initiation
    AuthorisationInitiation,
    /// cain.003 — Financial presentment
    FinancialInitiation,
    /// cain.006 — Reversal
    ReversalInitiation,
    /// cain.016 — Inquiry
    InquiryInitiation,
    /// cain.023 — Card management
    CardManagementInitiation,
    /// caad.008 — Administrative
    AdministrativeInitiation,
}

impl PayloadType {
    /// Returns the ISO 20022 message code (e.g., "pain.001").
    pub fn message_code(self) -> &'static str {
        match self {
            Self::CreditTransferInitiation => "pain.001",
            Self::FiToFiPaymentStatusReport => "pacs.002",
            Self::PaymentReturn => "pacs.004",
            Self::FiToFiCustomerCreditTransfer => "pacs.008",
            Self::PaymentStatusRequest => "pacs.028",
            Self::ResolutionOfInvestigation => "camt.029",
            Self::FiToFiPaymentCancellationRequest => "camt.056",
            Self::ReceiptAcknowledgement => "admi.007",
            Self::AuthorisationInitiation => "cain.001",
            Self::FinancialInitiation => "cain.003",
            Self::ReversalInitiation => "cain.006",
            Self::InquiryInitiation => "cain.016",
            Self::CardManagementInitiation => "cain.023",
            Self::AdministrativeInitiation => "caad.008",
        }
    }

    /// Returns true if this is a Card payload type.
    pub fn is_card(self) -> bool {
        matches!(
            self,
            Self::AuthorisationInitiation
                | Self::FinancialInitiation
                | Self::ReversalInitiation
                | Self::InquiryInitiation
                | Self::CardManagementInitiation
                | Self::AdministrativeInitiation
        )
    }

    /// Returns true if this is an A2A payload type.
    pub fn is_a2a(self) -> bool {
        !self.is_card()
    }

    /// Parse from a string (case-insensitive, accepts both enum name and
    /// ISO message code).
    pub fn from_str_loose(s: &str) -> Option<Self> {
        let lower = s.to_ascii_lowercase();
        // Match ISO message codes first.
        let val = match lower.as_str() {
            "pain.001" => Self::CreditTransferInitiation,
            "pacs.002" => Self::FiToFiPaymentStatusReport,
            "pacs.004" => Self::PaymentReturn,
            "pacs.008" => Self::FiToFiCustomerCreditTransfer,
            "pacs.028" => Self::PaymentStatusRequest,
            "camt.029" => Self::ResolutionOfInvestigation,
            "camt.056" => Self::FiToFiPaymentCancellationRequest,
            "admi.007" => Self::ReceiptAcknowledgement,
            "cain.001" => Self::AuthorisationInitiation,
            "cain.003" => Self::FinancialInitiation,
            "cain.006" => Self::ReversalInitiation,
            "cain.016" => Self::InquiryInitiation,
            "cain.023" => Self::CardManagementInitiation,
            "caad.008" => Self::AdministrativeInitiation,
            _ => return None,
        };
        Some(val)
    }
}

/// A typed ISO 20022 payload wrapper.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Iso20022Payload {
    /// The ISO 20022 message type.
    pub payload_type: PayloadType,
    /// Correlation id for this message instance.
    pub message_id: String,
    /// The message body as JSON.
    pub payload: serde_json::Value,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_codes() {
        assert_eq!(PayloadType::CreditTransferInitiation.message_code(), "pain.001");
        assert_eq!(PayloadType::FiToFiPaymentStatusReport.message_code(), "pacs.002");
        assert_eq!(PayloadType::PaymentReturn.message_code(), "pacs.004");
        assert_eq!(PayloadType::FiToFiCustomerCreditTransfer.message_code(), "pacs.008");
        assert_eq!(PayloadType::PaymentStatusRequest.message_code(), "pacs.028");
        assert_eq!(PayloadType::ResolutionOfInvestigation.message_code(), "camt.029");
        assert_eq!(PayloadType::FiToFiPaymentCancellationRequest.message_code(), "camt.056");
        assert_eq!(PayloadType::ReceiptAcknowledgement.message_code(), "admi.007");
        assert_eq!(PayloadType::AuthorisationInitiation.message_code(), "cain.001");
        assert_eq!(PayloadType::FinancialInitiation.message_code(), "cain.003");
        assert_eq!(PayloadType::ReversalInitiation.message_code(), "cain.006");
        assert_eq!(PayloadType::InquiryInitiation.message_code(), "cain.016");
        assert_eq!(PayloadType::CardManagementInitiation.message_code(), "cain.023");
        assert_eq!(PayloadType::AdministrativeInitiation.message_code(), "caad.008");
    }

    #[test]
    fn test_is_card_is_a2a() {
        assert!(PayloadType::AuthorisationInitiation.is_card());
        assert!(PayloadType::FinancialInitiation.is_card());
        assert!(!PayloadType::CreditTransferInitiation.is_card());
        assert!(PayloadType::CreditTransferInitiation.is_a2a());
        assert!(!PayloadType::AuthorisationInitiation.is_a2a());
    }

    #[test]
    fn test_from_str_loose() {
        assert_eq!(
            PayloadType::from_str_loose("pain.001"),
            Some(PayloadType::CreditTransferInitiation)
        );
        assert_eq!(
            PayloadType::from_str_loose("PACS.008"),
            Some(PayloadType::FiToFiCustomerCreditTransfer)
        );
        assert_eq!(
            PayloadType::from_str_loose("cain.001"),
            Some(PayloadType::AuthorisationInitiation)
        );
        assert_eq!(PayloadType::from_str_loose("unknown"), None);
    }

    #[test]
    fn test_serde_roundtrip() {
        let pt = PayloadType::FiToFiCustomerCreditTransfer;
        let json = serde_json::to_string(&pt).unwrap();
        assert_eq!(json, "\"FI_TO_FI_CUSTOMER_CREDIT_TRANSFER\"");
        let de: PayloadType = serde_json::from_str(&json).unwrap();
        assert_eq!(de, pt);
    }

    #[test]
    fn test_iso_payload_serde() {
        let p = Iso20022Payload {
            payload_type: PayloadType::CreditTransferInitiation,
            message_id: "MSG-001".into(),
            payload: serde_json::json!({"amount": 100.0}),
        };
        let json = serde_json::to_string(&p).unwrap();
        assert!(json.contains("\"payload_type\":\"CREDIT_TRANSFER_INITIATION\""));
        assert!(json.contains("\"message_id\":\"MSG-001\""));
        let de: Iso20022Payload = serde_json::from_str(&json).unwrap();
        assert_eq!(de.payload_type, PayloadType::CreditTransferInitiation);
    }
}