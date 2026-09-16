use std::error::Error;

use serde::{Deserialize, Serialize};

use crate::models::UInt64;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "status", content = "response", rename_all = "camelCase")]
pub enum TransactionBroadcastResponse {
    Ok(ExchangeResponse),
    Err(String),
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data", rename_all = "camelCase")]
pub enum ExchangeResponse {
    Default,
    Order(OrderData),
    Cancel(OrderData),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderData {
    pub statuses: Vec<OrderStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum OrderStatus {
    Success,
    WaitingForFill,
    WaitingForTrigger,
    Filled(OrderId),
    Resting(OrderId),
    Error(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderId {
    pub oid: UInt64,
}

impl TransactionBroadcastResponse {
    pub fn into_result(self) -> Result<Option<UInt64>, Box<dyn Error + Send + Sync>> {
        let data = match self {
            Self::Ok(ExchangeResponse::Default) => return Ok(None),
            Self::Ok(ExchangeResponse::Order(data) | ExchangeResponse::Cancel(data)) => data,
            Self::Err(error) => return Err(error.into()),
            Self::Error => return Err("Request failed".into()),
        };
        if data.statuses.is_empty() {
            return Err("Missing HyperCore action status".into());
        }
        data.statuses.into_iter().try_fold(None, |order_id, status| match status {
            OrderStatus::Filled(order) | OrderStatus::Resting(order) => Ok(order_id.or(Some(order.oid))),
            OrderStatus::Success | OrderStatus::WaitingForFill | OrderStatus::WaitingForTrigger => Ok(order_id),
            OrderStatus::Error(error) => Err(error.into()),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_into_result() {
        for (response, expected) in [
            (include_str!("../../testdata/order_broadcast_filled.json"), Ok(Some(134896397196))),
            (include_str!("../../testdata/order_broadcast_resting.json"), Ok(Some(789012))),
            (
                include_str!("../../testdata/order_broadcast_error.json"),
                Err("Reduce only order would increase position. asset=159"),
            ),
            (include_str!("../../testdata/order_broadcast_simple_error.json"), Err("Request failed")),
            (
                include_str!("../../testdata/transaction_broadcast_error_extra_agent.json"),
                Err("Extra agent already used."),
            ),
            (r#"{"status":"ok","response":{"type":"default"}}"#, Ok(None)),
            (r#"{"status":"ok","response":{"type":"cancel","data":{"statuses":["success"]}}}"#, Ok(None)),
            (
                r#"{"status":"ok","response":{"type":"order","data":{"statuses":["waitingForTrigger","waitingForFill"]}}}"#,
                Ok(None),
            ),
            (
                r#"{"status":"ok","response":{"type":"order","data":{"statuses":[]}}}"#,
                Err("Missing HyperCore action status"),
            ),
        ] {
            let result = serde_json::from_str::<TransactionBroadcastResponse>(response).unwrap().into_result();
            assert_eq!(result.map_err(|error| error.to_string()), expected.map_err(str::to_string));
        }
    }

    #[test]
    fn test_deserialize() {
        for response in [
            r#"{"status":"ok"}"#,
            r#"{"status":"ok","response":{"type":"order"}}"#,
            r#"{"status":"ok","response":{"type":"order","data":{"statuses":["unknown"]}}}"#,
            r#"{"status":"ok","response":{"type":"unknown"}}"#,
        ] {
            assert!(serde_json::from_str::<TransactionBroadcastResponse>(response).is_err());
        }
    }
}
