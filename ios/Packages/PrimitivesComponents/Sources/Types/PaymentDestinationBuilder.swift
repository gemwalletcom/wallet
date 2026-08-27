// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation
import Gemstone
import GemstonePrimitives
import Localization
import Primitives

public enum PaymentDestinationBuilder {
    public enum TransferDestination: Sendable {
        case confirm(TransferData)
        case recipient(RecipientData)
    }

    public static func transfer(payment: Primitives.PaymentRequest, asset: Primitives.Asset) throws -> TransferDestination {
        switch try Gemstone.paymentTransferDestination(request: payment.json(), asset: asset.paymentWalletAsset) {
        case let .confirm(transfer):
            return try .confirm(transferData(transfer: transfer, asset: asset))
        case .recipient:
            return .recipient(recipientData(for: payment, chain: asset.chain))
        case .selectAsset, .unsupported:
            throw AnyError(Localized.Errors.notSupported)
        }
    }

    public static func build(payment: Primitives.PaymentRequest, assets: [AssetData]) throws -> PaymentDestination {
        switch try Gemstone.paymentDestination(request: payment.json(), assets: assets.map { $0.asset.paymentWalletAsset }) {
        case let .confirm(transfer):
            guard let assetData = assetData(for: transfer.assetId, in: assets) else {
                throw AnyError(Localized.Errors.notSupported)
            }
            return try .confirm(transferData(transfer: transfer, asset: assetData.asset))
        case let .recipient(assetId):
            guard let assetData = assetData(for: assetId, in: assets) else {
                throw AnyError(Localized.Errors.notSupported)
            }
            return .recipient(
                SelectedAssetInput(
                    type: .send(.asset(assetData.asset)),
                    assetData: assetData,
                    recipient: recipientData(for: payment, chain: assetData.asset.chain),
                ),
            )
        case let .selectAsset(chains):
            return .selectAsset(.send(recipientData(for: payment)), chains: chains.compactMap { Primitives.Chain(rawValue: $0) })
        case .unsupported:
            throw AnyError(Localized.Errors.notSupported)
        }
    }

    public static func build(transaction: GemPaymentTransaction, asset: Primitives.Asset) throws -> PaymentDestination {
        let type = try TransferDataType.generic(
            asset: asset,
            metadata: Primitives.ApplicationMetadata(transaction.merchant),
            extra: TransferDataExtra(
                to: transaction.request
                    .map { try Primitives.PaymentRequest($0).address }
                    .map { asset.chain.checksumAddress($0) } ?? "",
                data: Data(transaction.transaction.utf8),
                outputType: .encodedTransaction,
                outputAction: .send,
                transactionType: Primitives.TransactionType(transaction.transactionType),
            ),
        )
        let transfer = transaction.request.flatMap {
            Gemstone.paymentDecodedTransfer(request: $0, asset: asset.paymentWalletAsset)
        }
        guard let transfer else {
            return .confirm(
                TransferData(
                    type: type,
                    recipient: Recipient(name: nil, address: "", memo: transaction.memo),
                    value: .zero,
                ),
            )
        }
        return try .confirm(
            TransferData(
                type: type,
                recipient: transferData(transfer: transfer, asset: asset).recipient,
                value: BigInt.from(string: transfer.value),
            ),
        )
    }

    private static func assetData(for assetId: String, in assets: [AssetData]) -> AssetData? {
        assets.first { $0.asset.id.identifier == assetId }
    }

    private static func transferData(transfer: GemPaymentConfirmTransfer, asset: Primitives.Asset) throws -> TransferData {
        try TransferData(
            type: .transfer(asset),
            recipient: Recipient(name: nil, address: transfer.address, memo: transfer.memo, references: transfer.references),
            value: BigInt.from(string: transfer.value),
        )
    }

    private static func recipientData(for payment: Primitives.PaymentRequest, chain: Primitives.Chain? = nil) -> RecipientData {
        let address = chain.map { $0.checksumAddress(payment.address) } ?? payment.address
        return RecipientData(
            recipient: Recipient(name: .none, address: address, memo: payment.memo, references: payment.references ?? []),
            amount: payment.exactAmount,
        )
    }
}

public extension Primitives.Asset {
    var paymentWalletAsset: GemPaymentWalletAsset {
        GemPaymentWalletAsset(assetId: id.identifier, decimals: decimals)
    }
}
