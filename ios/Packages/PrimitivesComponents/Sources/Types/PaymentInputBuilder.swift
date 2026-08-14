// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Localization
import Primitives

public enum PaymentInputBuilder {
    public static func build(payment: PaymentRequest, assets: [AssetData]) throws -> PaymentInput {
        switch PaymentAsset.from(payment, assets: assets) {
        case .unsupported:
            throw AnyError(Localized.Errors.notSupported)
        case let .single(assetData):
            return try build(payment: payment, assetData: assetData)
        case let .choice(payable):
            return .selectAsset(.send(recipientData(for: payment)), chains: payable.map(\.asset.chain))
        }
    }

    private static func build(payment: PaymentRequest, assetData: AssetData) throws -> PaymentInput {
        switch try PaymentTransfer(asset: assetData.asset).destination(for: payment) {
        case let .confirm(transfer):
            return .confirm(transfer)
        case let .recipient(recipient):
            return .recipient(
                SelectedAssetInput(
                    type: .send(.asset(assetData.asset)),
                    assetData: assetData,
                    recipient: recipient,
                ),
            )
        }
    }

    private static func recipientData(for payment: PaymentRequest) -> RecipientData {
        let amount: String? = switch payment.amount {
        case let .exactValue(value): value
        case .atomicValue, .none: .none
        }
        return RecipientData(
            recipient: Recipient(name: .none, address: payment.address, memo: payment.memo),
            amount: amount,
        )
    }
}
