// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import struct Gemstone.GemAcquireAsset
import struct Gemstone.GemConfirmErrorInfo
import GemstonePrimitives
import InfoSheet
import Localization
import Primitives
import PrimitivesComponents

enum ConfirmInfoSheetBuilder {
    static func build(
        for info: GemConfirmErrorInfo,
        onGetAsset: @escaping @MainActor @Sendable (Asset, GemAcquireAsset) -> Void,
    ) -> InfoSheetType {
        let asset = info.asset?.toPrimitives()
        let image = asset.map { AssetIdViewModel(assetId: $0.id).assetImage } ?? AssetImage()
        let button = acquireButton(info, asset: asset, onGetAsset: onGetAsset)

        return switch info.sheet {
        case .balanceRequired: .balanceRequired(info, image: image, button: button)
        case .networkFeeRequired, .networkFeeMissing: .insufficientNetworkFee(info, image: image, button: button)
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
        onGetAsset: @escaping @MainActor @Sendable (Asset, GemAcquireAsset) -> Void,
    ) -> InfoSheetButton? {
        guard let asset, let acquire = info.acquire else { return nil }
        return .action(title: acquire.flow.actionTitle(symbol: asset.symbol), action: { onGetAsset(asset, acquire) })
    }
}
