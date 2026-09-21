// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import enum Gemstone.GemAcquireAssetFlow
import struct Gemstone.GemConfirmErrorInfo
import GemstonePrimitives
import InfoSheet
import Localization
import Primitives
import PrimitivesComponents

enum ConfirmInfoSheetBuilder {
    static func build(
        for info: GemConfirmErrorInfo,
        networkFeeBuyAmount: Int,
        onGetAsset: @escaping @MainActor @Sendable (Asset, Int?) -> Void,
    ) -> InfoSheetType {
        let asset = info.asset?.toPrimitives()
        let image = asset.map { AssetViewModel(asset: $0).assetImage } ?? AssetImage()
        let button = acquireButton(info, asset: asset, buyAmount: nil, onGetAsset: onGetAsset)
        let feeButton = acquireButton(info, asset: asset, buyAmount: networkFeeBuyAmount, onGetAsset: onGetAsset)

        return switch info.sheet {
        case .balanceRequired: .balanceRequired(info, image: image, button: button)
        case .networkFeeRequired, .networkFeeMissing: .insufficientNetworkFee(info, image: image, button: feeButton)
        case .minimumAccountBalance: .accountMinimalBalance(info)
        case let .swapMinimum(provider, providerName):
            .swapMinimumAmount(info, providerName: providerName, image: AssetImage(placeholder: provider.toPrimitives().image), button: button)
        case let .dustThreshold(chain):
            .dustThreshold(Chain(core: chain), image: AssetViewModel(asset: Chain(core: chain).asset).assetImage)
        case .malicious: .maliciousTransaction
        case let .memoRequired(symbol): .memoRequired(symbol: symbol)
        }
    }

    private static func acquireButton(
        _ info: GemConfirmErrorInfo,
        asset: Asset?,
        buyAmount: Int?,
        onGetAsset: @escaping @MainActor @Sendable (Asset, Int?) -> Void,
    ) -> InfoSheetButton? {
        guard let asset, let flow = info.acquire else { return nil }
        return .action(title: flow.actionTitle(symbol: asset.symbol), action: { onGetAsset(asset, buyAmount) })
    }
}
