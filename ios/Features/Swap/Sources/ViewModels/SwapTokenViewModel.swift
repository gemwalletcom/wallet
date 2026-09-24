// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Formatters
import Foundation
import Gemstone
import struct Gemstone.GemSwapSideInteraction
import GemstonePrimitives
import Localization
import Primitives
import PrimitivesComponents

enum SwapTokenViewType {
    case selected(AssetDataViewModel)
    case placeholder
}

struct SwapTokenViewModel {
    private let type: SwapTokenViewType
    let interaction: GemSwapSideInteraction

    init(
        type: SwapTokenViewType,
        interaction: GemSwapSideInteraction,
    ) {
        self.type = type
        self.interaction = interaction
    }

    var availableBalanceText: String? {
        switch type {
        case let .selected(model):
            Gemstone.availableBalanceText(
                asset: model.asset.toGem(),
                balance: GemAssetBalance(model.assetData.balance, assetId: model.asset.id, isActive: model.assetData.metadata.isActive),
            ).text
        case .placeholder: nil
        }
    }

    var isBalanceDisabled: Bool {
        availableBalanceText == nil || !interaction.isBalanceActionEnabled
    }

    var assetImage: AssetImage? {
        switch type {
        case let .selected(model): model.assetImage
        case .placeholder: nil
        }
    }

    var actionTitle: String {
        switch type {
        case let .selected(model): model.asset.symbol
        case .placeholder: Localized.Assets.selectAsset
        }
    }

    var amountPlaceholder: String {
        switch type {
        case .selected: .zero
        case .placeholder: .empty
        }
    }

    func fiatBalance(amount: String) -> String? {
        switch type {
        case let .selected(model):
            guard let value = try? NumberInput.value(amount, decimals: model.asset.decimals.asInt) else { return nil }
            return model.priceViewModel.fiatValueText(value: value, decimals: model.asset.decimals.asInt)
        case .placeholder:
            return nil
        }
    }
}
