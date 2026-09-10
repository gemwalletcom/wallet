// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Gemstone
import protocol Gemstone.GemPaymentServiceProtocol
import GemstonePrimitives
import Localization
import Primitives
import PrimitivesComponents

public enum PaymentDestinationBuilder {
    public enum TransferDestination: Sendable {
        case confirm(GemTransferData)
        case recipient(GemPaymentRecipient)
    }

    public static func transfer(
        payment: Gemstone.PaymentRequest,
        asset: Primitives.Asset,
        paymentService: any GemPaymentServiceProtocol,
    ) throws -> TransferDestination {
        switch paymentService.transferDestination(request: payment, asset: asset.paymentWalletAsset) {
        case let .confirm(transfer):
            return .confirm(paymentService.transferData(transfer: transfer, asset: asset.map()))
        case let .recipient(_, payment):
            return .recipient(payment)
        case .selectAsset, .unsupported:
            throw AnyError(Localized.Errors.notSupported)
        }
    }

    public static func build(
        payment: Gemstone.PaymentRequest,
        assets: [AssetData],
        paymentService: any GemPaymentServiceProtocol,
    ) throws -> PaymentDestination {
        switch paymentService.destination(request: payment, assets: assets.map(\.asset.paymentWalletAsset)) {
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
                    type: .send(.asset(asset: assetData.asset.map())),
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

    private static func assetData(for assetId: String, in assets: [AssetData]) -> AssetData? {
        assets.first { $0.asset.id.identifier == assetId }
    }
}

public extension Primitives.Asset {
    var paymentWalletAsset: GemPaymentWalletAsset {
        GemPaymentWalletAsset(assetId: id.identifier, decimals: decimals)
    }
}
