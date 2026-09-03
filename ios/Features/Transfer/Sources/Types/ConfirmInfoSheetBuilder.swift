// Copyright (c). Gem Wallet. All rights reserved.

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
        feePrice: Price?,
        currency: String,
        onGetAsset: @escaping @MainActor @Sendable (Asset, Int?) -> Void,
    ) -> InfoSheetType? {
        switch error {
        case let .confirm(error):
            confirmSheet(for: error, feePrice: feePrice, currency: currency, onGetAsset: onGetAsset)
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
        case let .InsufficientBalance(asset, requirement):
            let asset = asset.map()
            return .balanceRequired(asset, image: image(for: asset), requirement: requirement.map(), action: { onGetAsset(asset, nil) })
        case let .InsufficientNetworkFee(asset, requirement):
            let asset = asset.map()
            return .insufficientNetworkFee(asset, image: image(for: asset), requirement: requirement?.map(), price: feePrice, currency: currency, action: {
                onGetAsset(asset, FiatConfig.insufficientNetworkFeeBuyAmount)
            })
        case let .MinimumAccountBalanceTooLow(asset, requirement):
            return .accountMinimalBalance(asset.map(), required: requirement.required)
        case let .Sign(.dustThreshold, chain, _):
            let chain = Chain(core: chain)
            return .dustThreshold(chain, image: image(for: chain.asset))
        case .ScanMalicious: return .maliciousTransaction
        case let .ScanMemoRequired(symbol): return .memoRequired(symbol: symbol)
        default: return nil
        }
    }

    private static func image(for asset: Asset) -> AssetImage {
        AssetViewModel(asset: asset).assetImage
    }
}
