// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.GemCandleTooltipRow
import enum Gemstone.GemPerpetualButton
import enum Gemstone.GemPerpetualChartLineKind
import enum Gemstone.GemPerpetualInfoRow
import enum Gemstone.GemPerpetualMarketSection
import enum Gemstone.GemPerpetualPositionDetailRow
import enum Gemstone.GemPerpetualSection
import Localization
import Primitives

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

extension GemPerpetualPositionDetailRow {
    var title: String {
        switch self {
        case .pnl: Localized.Perpetual.pnl
        case .autoclose: Localized.Perpetual.autoClose
        case .size: Localized.Perpetual.size
        case .entryPrice: Localized.Perpetual.entryPrice
        case .liquidationPrice: Localized.Info.Perpetual.LiquidationPrice.title
        case .margin: Localized.Perpetual.margin
        case .fundingPayments: Localized.Info.Perpetual.FundingPayments.title
        }
    }
}

extension GemPerpetualInfoRow {
    var title: String {
        switch self {
        case .dailyVolume: Localized.Markets.dailyVolume
        case .openInterest: Localized.Info.Perpetual.OpenInterest.title
        case .fundingRate: Localized.Info.Perpetual.FundingApr.title
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
