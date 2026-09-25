// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import struct Gemstone.GemSwapSideInteraction
import struct Gemstone.GemSwapSideState
import GemstonePrimitives
import Localization
import Primitives
import PrimitivesComponents

struct SwapTokenViewModel {
    private let asset: Asset?
    private let side: GemSwapSideState

    init(asset: Asset?, side: GemSwapSideState) {
        self.asset = asset
        self.side = side
    }

    var interaction: GemSwapSideInteraction {
        side.interaction
    }

    var availableBalanceText: String? {
        side.balance?.text
    }

    var fiatText: String? {
        side.fiat?.text()
    }

    var isBalanceDisabled: Bool {
        !side.interaction.isBalanceActionEnabled
    }

    var assetImage: AssetImage? {
        asset.map { AssetIdViewModel(assetId: $0.id).assetImage }
    }

    var actionTitle: String {
        asset?.symbol ?? Localized.Assets.selectAsset
    }

    var amountPlaceholder: String {
        asset == nil ? .empty : .zero
    }
}
