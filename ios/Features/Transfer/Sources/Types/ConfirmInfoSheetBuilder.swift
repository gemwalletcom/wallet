// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import enum Gemstone.GemAcquireAssetFlow
import enum Gemstone.GemConfirmError
import GemstonePrimitives
import InfoSheet
import Localization
import Primitives
import PrimitivesComponents

enum ConfirmInfoSheetBuilder {
    static func build(
        for error: ConfirmTransferError,
        feePrice: Price?,
        currency: String,
        acquireFlow: (Asset) -> GemAcquireAssetFlow,
        onGetAsset: @escaping @MainActor @Sendable (Asset, Int?) -> Void,
    ) -> InfoSheetType? {
        switch error {
        case let .confirm(error):
            confirmSheet(for: error, feePrice: feePrice, currency: currency, acquireFlow: acquireFlow, onGetAsset: onGetAsset)
        case .other:
            nil
        }
    }

    private static func confirmSheet(
        for error: GemConfirmError,
        feePrice: Price?,
        currency: String,
        acquireFlow: (Asset) -> GemAcquireAssetFlow,
        onGetAsset: @escaping @MainActor @Sendable (Asset, Int?) -> Void,
    ) -> InfoSheetType? {
        switch error {
        case let .InsufficientBalance(asset, requirement):
            let asset = asset.map()
            return .balanceRequired(asset, image: image(for: asset), requirement: requirement.map(), button: acquireButton(asset, flow: acquireFlow(asset)) { onGetAsset(asset, nil) })
        case let .InsufficientNetworkFee(asset, requirement):
            let asset = asset.map()
            return .insufficientNetworkFee(asset, image: image(for: asset), requirement: requirement?.map(), price: feePrice, currency: currency, button: acquireButton(asset, flow: acquireFlow(asset)) {
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

    private static func acquireButton(_ asset: Asset, flow: GemAcquireAssetFlow, action: @escaping InfoSheetAction) -> InfoSheetButton {
        switch flow {
        case .options: .action(title: Localized.Asset.getAsset(asset.symbol), action: action)
        case .fiat: .action(title: Localized.Asset.buyAsset(asset.symbol), action: action)
        }
    }
}
