// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import enum Gemstone.AutocloseValidation
import enum Gemstone.GemCandleTooltipRow
import enum Gemstone.GemPerpetualButton
import enum Gemstone.GemPerpetualChartLineKind
import enum Gemstone.GemPerpetualMarketSection
import enum Gemstone.GemPerpetualSection
import Localization
import Primitives

extension AutocloseValidation: @retroactive LocalizedError {
    public var errorDescription: String? {
        switch self {
        case .valid: nil
        case .invalidAmount: Localized.Errors.invalidAmount
        case .triggerMustBeHigher: Localized.Errors.Perpetual.triggerPriceHigher
        case .triggerMustBeLower: Localized.Errors.Perpetual.triggerPriceLower
        }
    }
}

extension TpslType {
    var autocloseTitle: String {
        switch self {
        case .takeProfit: Localized.Perpetual.AutoClose.takeProfit
        case .stopLoss: Localized.Perpetual.AutoClose.stopLoss
        }
    }
}

extension GemPerpetualSection {
    var title: String {
        switch self {
        case .position: Localized.Perpetual.position
        case .info: Localized.Common.info
        }
    }
}

extension GemPerpetualButton {
    var title: String {
        switch self {
        case .long: Localized.Perpetual.long
        case .short: Localized.Perpetual.short
        case .modify: Localized.Perpetual.modify
        case .close: Localized.Perpetual.closePosition
        case .increase: Localized.Perpetual.increasePosition
        case .reduce: Localized.Perpetual.reducePosition
        }
    }
}

extension GemPerpetualChartLineKind {
    var title: String {
        switch self {
        case .takeProfit: Localized.Perpetual.takeProfit
        case .stopLoss: Localized.Perpetual.stopLoss
        case .entry: Localized.Charts.entry
        case .liquidation: Localized.Perpetual.liquidation
        }
    }
}

extension GemCandleTooltipRow {
    var title: String {
        switch self {
        case .open: Localized.Charts.Price.open
        case .high: Localized.Charts.Price.high
        case .low: Localized.Charts.Price.low
        case .close: Localized.Charts.Price.close
        case .change: Localized.Charts.Price.change
        case .volume: Localized.Perpetual.volume
        }
    }
}

extension GemPerpetualMarketSection {
    var title: String {
        switch self {
        case .positions: Localized.Perpetual.positions
        case .pinned: Localized.Common.pinned
        case .markets: Localized.Perpetuals.markets
        case .recents, .empty: .empty
        }
    }
}
