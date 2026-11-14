use chrono::DateTime;
use spark_wallet::{SspUserRequest, TransferDirection, TransferStatus, TransferType, WalletTransfer};

use crate::model::{LightningPayment, LightningPaymentDirection, LightningPaymentStatus};

pub fn map_transfer(transfer: &WalletTransfer) -> Option<LightningPayment> {
    if !is_payment(transfer.transfer_type) {
        return None;
    }
    let transfer_status = payment_status(transfer.status, &transfer.direction);
    let (status, fee_sats, invoice) = match &transfer.user_request {
        Some(SspUserRequest::LightningSendRequest(request)) => (
            request.lightning_send_payment_preimage.as_ref().map_or(transfer_status, |_| LightningPaymentStatus::Completed),
            request.fee.as_sats().unwrap_or_default(),
            Some(request.encoded_invoice.clone()),
        ),
        Some(SspUserRequest::LightningReceiveRequest(request)) => (transfer_status, 0, Some(request.invoice.encoded_invoice.clone())),
        Some(SspUserRequest::CoopExitRequest(_)) | Some(SspUserRequest::ClaimStaticDeposit(_)) | Some(SspUserRequest::LeavesSwapRequest(_)) | None => (transfer_status, 0, None),
    };
    Some(LightningPayment {
        id: transfer.id.to_string(),
        direction: payment_direction(&transfer.direction),
        status,
        amount_sats: transfer.total_value_sat.saturating_sub(fee_sats),
        fee_sats,
        created_at: transfer.created_at.map(DateTime::from).unwrap_or_default(),
        invoice,
    })
}

fn is_payment(transfer_type: TransferType) -> bool {
    match transfer_type {
        TransferType::PreimageSwap | TransferType::Transfer | TransferType::UtxoSwap => true,
        TransferType::CooperativeExit | TransferType::Swap | TransferType::CounterSwap | TransferType::PrimarySwapV3 | TransferType::CounterSwapV3 => false,
    }
}

fn payment_status(status: TransferStatus, direction: &TransferDirection) -> LightningPaymentStatus {
    match status {
        TransferStatus::Completed => LightningPaymentStatus::Completed,
        TransferStatus::SenderKeyTweaked => match direction {
            TransferDirection::Outgoing => LightningPaymentStatus::Completed,
            TransferDirection::Incoming => LightningPaymentStatus::Pending,
        },
        TransferStatus::Expired | TransferStatus::Returned => LightningPaymentStatus::Failed,
        TransferStatus::SenderInitiated
        | TransferStatus::SenderKeyTweakPending
        | TransferStatus::ReceiverKeyTweaked
        | TransferStatus::ReceiverRefundSigned
        | TransferStatus::SenderInitiatedCoordinator
        | TransferStatus::ReceiverKeyTweakLocked
        | TransferStatus::ReceiverKeyTweakApplied
        | TransferStatus::ApplyingSenderKeyTweak => LightningPaymentStatus::Pending,
    }
}

fn payment_direction(direction: &TransferDirection) -> LightningPaymentDirection {
    match direction {
        TransferDirection::Incoming => LightningPaymentDirection::Incoming,
        TransferDirection::Outgoing => LightningPaymentDirection::Outgoing,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_payment_status() {
        assert_eq!(payment_status(TransferStatus::Completed, &TransferDirection::Incoming), LightningPaymentStatus::Completed);
        assert_eq!(payment_status(TransferStatus::SenderKeyTweaked, &TransferDirection::Outgoing), LightningPaymentStatus::Completed);
        assert_eq!(payment_status(TransferStatus::SenderKeyTweaked, &TransferDirection::Incoming), LightningPaymentStatus::Pending);
        assert_eq!(payment_status(TransferStatus::Expired, &TransferDirection::Outgoing), LightningPaymentStatus::Failed);
        assert_eq!(payment_status(TransferStatus::Returned, &TransferDirection::Incoming), LightningPaymentStatus::Failed);
        assert_eq!(payment_status(TransferStatus::ReceiverKeyTweakLocked, &TransferDirection::Incoming), LightningPaymentStatus::Pending);
    }
}
