// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Gemstone
import class Gemstone.GemPaymentService
import GemstonePrimitives
import Localization
import Primitives

public enum PaymentDestinationBuilder {
    public enum TransferDestination: Sendable {
        case confirm(GemTransferData)
        case recipient(GemPaymentRecipient)
    }

    public static func transfer(
        payment: Primitives.PaymentRequest,
        asset: Primitives.Asset,
        paymentService: GemPaymentService,
    ) throws -> TransferDestination {
        switch paymentService.transferDestination(request: payment.json(), asset: asset.paymentWalletAsset) {
        case let .confirm(transfer):
            return .confirm(paymentService.transferData(transfer: transfer, asset: asset.map()))
        case let .recipient(_, payment):
            return .recipient(payment)
        case .selectAsset, .unsupported:
            throw AnyError(Localized.Errors.notSupported)
        }
    }

    public static func build(
        payment: Primitives.PaymentRequest,
        assets: [AssetData],
        paymentService: GemPaymentService,
    ) throws -> PaymentDestination {
        switch paymentService.destination(request: payment.json(), assets: assets.map { $0.asset.paymentWalletAsset }) {
        case let .confirm(transfer):
            guard let assetData = assetData(for: transfer.assetId, in: assets) else {
                throw AnyError(Localized.Errors.notSupported)
            }
            return .confirm(paymentService.transferData(transfer: transfer, asset: assetData.asset.map()))
        case let .recipient(assetId, payment):
            guard let assetData = assetData(for: assetId, in: assets) else {
                throw AnyError(Localized.Errors.notSupported)
            }
            return .recipient(
                SelectedAssetInput(
                    type: .send(.asset(assetData.asset)),
                    assetData: assetData,
                    recipient: payment,
                ),
            )
        case let .selectAsset(payment, chains):
            return .selectAsset(.send(payment), chains: chains.compactMap { Primitives.Chain(rawValue: $0) })
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
}

public extension Primitives.Asset {
    var paymentWalletAsset: GemPaymentWalletAsset {
        GemPaymentWalletAsset(assetId: id.identifier, decimals: decimals)
    }
}
