// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Formatters
import Foundation
import Localization
import Primitives
import PrimitivesComponents

enum SwapTokenViewType {
    case selected(AssetDataViewModel)
    case placeholder(currencyCode: String)
}

struct SwapTokenInteraction {
    let isAmountEditable: Bool
    let isAssetSelectable: Bool
    let isBalanceActionEnabled: Bool

    static func pay(isEnabled: Bool) -> SwapTokenInteraction {
        SwapTokenInteraction(
            isAmountEditable: isEnabled,
            isAssetSelectable: isEnabled,
            isBalanceActionEnabled: isEnabled,
        )
    }

    static func receive(isEnabled: Bool) -> SwapTokenInteraction {
        SwapTokenInteraction(
            isAmountEditable: false,
            isAssetSelectable: isEnabled,
            isBalanceActionEnabled: false,
        )
    }
}

struct SwapTokenViewModel {
    private let type: SwapTokenViewType
    private let formatter = ValueFormatter(style: .medium)
    let interaction: SwapTokenInteraction

    init(
        type: SwapTokenViewType,
        interaction: SwapTokenInteraction,
    ) {
        self.type = type
        self.interaction = interaction
    }

    var availableBalanceText: String? {
        switch type {
        case let .selected(model): Localized.Transfer.balance(model.availableBalanceText)
        case .placeholder: nil
        }
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

    func fiatBalance(amount: String) -> String {
        switch type {
        case let .selected(model):
            guard
                let value = try? formatter.inputNumber(from: amount, decimals: model.asset.decimals.asInt),
                let amount = try? formatter.double(from: value, decimals: model.asset.decimals.asInt),
                let price = model.priceViewModel.price
            else {
                return .empty
            }
            return model.priceViewModel.fiatAmountText(amount: price.price * amount)
        case let .placeholder(currencyCode):
            return CurrencyFormatter(currencyCode: currencyCode).string(.zero)
        }
    }
}
