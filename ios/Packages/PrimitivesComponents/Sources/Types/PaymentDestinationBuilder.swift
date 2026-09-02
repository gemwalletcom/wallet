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
            return .confirm(paymentService.transferData(transfer: transfer, asset: asset.map()))
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
            return .confirm(paymentService.transferData(transfer: transfer, asset: assetData.asset.map()))
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
        paymentService: GemPaymentService,
    ) -> PaymentDestination {
        .confirm(paymentService.transactionTransferData(transaction: transaction, asset: asset.map()))
    }

    private static func assetData(for assetId: String, in assets: [AssetData]) -> AssetData? {
        assets.first { $0.asset.id.identifier == assetId }
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
