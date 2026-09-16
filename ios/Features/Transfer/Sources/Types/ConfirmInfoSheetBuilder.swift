// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import enum Gemstone.GemAcquireAssetFlow
import enum Gemstone.GemConfirmErrorDisplay
import GemstonePrimitives
import InfoSheet
import Localization
import Primitives
import PrimitivesComponents

enum ConfirmInfoSheetBuilder {
    static func build(
        for error: ConfirmTransferError,
        feePrice: Price?,
        prices: [AssetId: Price],
        currency: String,
        acquireFlow: (Asset) -> GemAcquireAssetFlow,
        networkFeeBuyAmount: Int,
        onGetAsset: @escaping @MainActor @Sendable (Asset, Int?) -> Void,
    ) -> InfoSheetType? {
        switch error {
        case let .confirm(error):
            confirmSheet(for: error.display(), feePrice: feePrice, prices: prices, currency: currency, acquireFlow: acquireFlow, networkFeeBuyAmount: networkFeeBuyAmount, onGetAsset: onGetAsset)
        case .other:
            nil
        }
    }

    private static func confirmSheet(
        for display: GemConfirmErrorDisplay,
        feePrice: Price?,
        prices: [AssetId: Price],
        currency: String,
        acquireFlow: (Asset) -> GemAcquireAssetFlow,
        networkFeeBuyAmount: Int,
        onGetAsset: @escaping @MainActor @Sendable (Asset, Int?) -> Void,
    ) -> InfoSheetType? {
        switch display {
        case let .balanceRequired(asset, requirement):
            let asset = asset.toPrimitives()
            return .balanceRequired(asset, image: image(for: asset), requirement: requirement.toPrimitives(), button: acquireButton(asset, flow: acquireFlow(asset)) { onGetAsset(asset, nil) })
        case let .networkFeeRequired(asset, requirement):
            let asset = asset.toPrimitives()
            return .insufficientNetworkFee(asset, image: image(for: asset), requirement: requirement.toPrimitives(), price: feePrice, currency: currency, button: acquireButton(asset, flow: acquireFlow(asset)) {
                onGetAsset(asset, networkFeeBuyAmount)
            })
        case let .networkFeeMissing(asset):
            let asset = asset.toPrimitives()
            return .insufficientNetworkFee(asset, image: image(for: asset), requirement: nil, price: feePrice, currency: currency, button: acquireButton(asset, flow: acquireFlow(asset)) {
                onGetAsset(asset, networkFeeBuyAmount)
            })
        case let .minimumAccountBalance(asset, required):
            return .accountMinimalBalance(asset.toPrimitives(), required: required)
        case let .swapMinimum(asset, provider, providerName, requirement):
            let asset = asset.toPrimitives()
            return .swapMinimumAmount(
                asset,
                providerName: providerName,
                image: AssetImage(placeholder: provider.toPrimitives().image),
                requirement: requirement.toPrimitives(),
                price: prices[asset.id],
                currency: currency,
                button: acquireButton(asset, flow: acquireFlow(asset)) { onGetAsset(asset, nil) },
            )
        case let .dustThreshold(chain):
            let chain = Chain(core: chain)
            return .dustThreshold(chain, image: image(for: chain.asset))
        case .malicious: return .maliciousTransaction
        case let .memoRequired(symbol): return .memoRequired(symbol: symbol)
        case .feeRatesMissing, .offline, .accountMissing, .unknown, .insufficientFunds, .cancelled, .message:
            return nil
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
