// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Gemstone
import GemstonePrimitives
import Localization
import Primitives

public enum PaymentDestinationBuilder {
    public static func build(payment: PaymentRequest, assets: [AssetData]) throws -> PaymentDestination {
        let payable = payableAssets(for: payment, in: assets)

        guard let assetData = payable.first else {
            throw AnyError(Localized.Errors.notSupported)
        }
        if payable.count > 1 {
            return .selectAsset(.send(recipientData(for: payment)), chains: payable.map(\.asset.chain).distinct())
        }
        return try build(payment: payment, assetData: assetData)
    }

    public static func build(transaction: GemPaymentTransaction, asset: Asset) throws -> PaymentDestination {
        let request: PaymentRequest? = try transaction.request?.map()
        let type = TransferDataType.generic(
            asset: asset,
            metadata: transaction.merchant.map(),
            extra: TransferDataExtra(
                to: request.map { asset.chain.checksumAddress($0.address) } ?? "",
                data: Data(transaction.transaction.utf8),
                outputType: .encodedTransaction,
                outputAction: .send,
                transactionType: transaction.transactionType.map(),
            ),
        )
        guard let request else {
            return .confirm(
                TransferData(
                    type: type,
                    recipientData: RecipientData(
                        recipient: Recipient(name: nil, address: "", memo: transaction.memo),
                        amount: nil,
                    ),
                    amount: .exact(.zero),
                ),
            )
        }
        guard let data = try PaymentTransfer(asset: asset).confirmation(for: request, type: type) else {
            throw AnyError(Localized.Errors.notSupported)
        }
        return .confirm(data)
    }

    private static func payableAssets(for payment: PaymentRequest, in assets: [AssetData]) -> [AssetData] {
        if let assetId = payment.assetId {
            return assets.filter { $0.asset.id == assetId }
        }
        return assets.filter { $0.asset.chain.isValidAddress(payment.address) }
    }

    private static func build(payment: PaymentRequest, assetData: AssetData) throws -> PaymentDestination {
        switch try PaymentTransfer(asset: assetData.asset).destination(for: payment) {
        case let .confirm(transfer):
            .confirm(transfer)
        case let .recipient(recipient):
            .recipient(
                SelectedAssetInput(
                    type: .send(.asset(assetData.asset)),
                    assetData: assetData,
                    recipient: recipient,
                ),
            )
        }
    }

    private static func recipientData(for payment: PaymentRequest) -> RecipientData {
        RecipientData(
            recipient: Recipient(name: .none, address: payment.address, memo: payment.memo),
            amount: payment.exactAmount,
        )
    }
}
