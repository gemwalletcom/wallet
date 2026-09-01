// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation
import Gemstone
import protocol Gemstone.GemAddressServiceProtocol
import class Gemstone.GemPaymentService
import GemstonePrimitives
import Localization
import Primitives

public enum PaymentDestinationBuilder {
    public enum TransferDestination: Sendable {
        case confirm(GemTransferData)
        case recipient(RecipientData)
    }

    public static func transfer(
        payment: Primitives.PaymentRequest,
        asset: Primitives.Asset,
        addressService: any GemAddressServiceProtocol,
        paymentService: GemPaymentService,
    ) throws -> TransferDestination {
        switch paymentService.transferDestination(request: payment.json(), asset: asset.paymentWalletAsset) {
        case let .confirm(transfer):
            return try .confirm(transferData(transfer: transfer, asset: asset))
        case .recipient:
            return .recipient(recipientData(for: payment, chain: asset.chain, addressService: addressService))
        case .selectAsset, .unsupported:
            throw AnyError(Localized.Errors.notSupported)
        }
    }

    public static func build(
        payment: Primitives.PaymentRequest,
        assets: [AssetData],
        addressService: any GemAddressServiceProtocol,
        paymentService: GemPaymentService,
    ) throws -> PaymentDestination {
        switch paymentService.destination(request: payment.json(), assets: assets.map { $0.asset.paymentWalletAsset }) {
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
                    recipient: recipientData(for: payment, chain: assetData.asset.chain, addressService: addressService),
                ),
            )
        case let .selectAsset(chains):
            return .selectAsset(.send(recipientData(for: payment, addressService: addressService)), chains: chains.compactMap { Primitives.Chain(rawValue: $0) })
        case .unsupported:
            throw AnyError(Localized.Errors.notSupported)
        }
    }

    public static func build(
        transaction: GemPaymentTransaction,
        asset: Primitives.Asset,
        addressService: any GemAddressServiceProtocol,
        paymentService: GemPaymentService,
    ) throws -> PaymentDestination {
        let type = try GemTransactionInputType.generic(
            asset: asset.map(),
            metadata: Primitives.ApplicationMetadata(transaction.merchant).json(),
            extra: TransferDataExtra(
                to: transaction.request
                    .map { try Primitives.PaymentRequest($0).address }
                    .map { asset.chain.checksumAddress($0, addressService: addressService) } ?? "",
                data: Data(transaction.transaction.utf8),
                outputType: .encodedTransaction,
                outputAction: .send,
                transactionType: Primitives.TransactionType(transaction.transactionType),
            ).map(),
        )
        let transfer = transaction.request.flatMap {
            paymentService.decodedTransfer(request: $0, asset: asset.paymentWalletAsset)
        }
        guard let transfer else {
            return .confirm(
                GemTransferData(
                    inputType: type,
                    recipient: GemRecipient(address: "", memo: transaction.memo),
                    value: BigInt.zero,
                ),
            )
        }
        return try .confirm(
            GemTransferData(
                inputType: type,
                recipient: transferData(transfer: transfer, asset: asset).recipient,
                value: transfer.value,
            ),
        )
    }

    private static func assetData(for assetId: String, in assets: [AssetData]) -> AssetData? {
        assets.first { $0.asset.id.identifier == assetId }
    }

    private static func transferData(transfer: GemPaymentConfirmTransfer, asset: Primitives.Asset) -> GemTransferData {
        GemTransferData(
            inputType: .transfer(asset: asset.map()),
            recipient: GemRecipient(address: transfer.address, memo: transfer.memo, references: transfer.references),
            value: transfer.value,
        )
    }

    private static func recipientData(for payment: Primitives.PaymentRequest, chain: Primitives.Chain? = nil, addressService: any GemAddressServiceProtocol) -> RecipientData {
        let address = chain.map { $0.checksumAddress(payment.address, addressService: addressService) } ?? payment.address
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
