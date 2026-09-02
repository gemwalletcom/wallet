// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Components
import Foundation
import GemstonePrimitives
import InfoSheet
import enum Gemstone.GemConfirmError
import Primitives
import PrimitivesComponents

enum ConfirmInfoSheetBuilder {
    static func build(
        for error: ConfirmTransferError,
        asset: Asset,
        feePrice: Price?,
        currency: String,
        onGetAsset: @escaping @MainActor @Sendable (Asset, Int?) -> Void,
    ) -> InfoSheetType? {
        switch error {
        case let .confirm(error):
            confirmSheet(for: error, feePrice: feePrice, currency: currency, onGetAsset: onGetAsset)
        case .chain(.dustThreshold):
            .dustThreshold(asset.chain, image: image(for: asset))
        case .chain, .other:
            nil
        }
    }

    private static func confirmSheet(
        for error: GemConfirmError,
        feePrice: Price?,
        currency: String,
        onGetAsset: @escaping @MainActor @Sendable (Asset, Int?) -> Void,
    ) -> InfoSheetType? {
        switch error {
        case let .InsufficientBalance(asset, required, available):
            let asset = asset.map()
            let requirement = BalanceRequirement(required: BigInt(core: required), available: BigInt(core: available))
            return .balanceRequired(asset, image: image(for: asset), requirement: requirement, action: { onGetAsset(asset, nil) })
        case let .InsufficientNetworkFee(asset, _, _):
            let asset = asset.map()
            return .insufficientNetworkFee(asset, image: image(for: asset), requirement: error.balanceRequirement, price: feePrice, currency: currency, action: {
                onGetAsset(asset, FiatConfig.insufficientNetworkFeeBuyAmount)
            })
        case let .MinimumAccountBalanceTooLow(asset, required, _):
            return .accountMinimalBalance(asset.map(), required: BigInt(core: required))
        case .ScanMalicious: return .maliciousTransaction
        case let .ScanMemoRequired(symbol): return .memoRequired(symbol: symbol)
        default: return nil
        }
    }

    private static func image(for asset: Asset) -> AssetImage {
        AssetViewModel(asset: asset).assetImage
    }
}
