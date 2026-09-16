// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.GemAssetMarketRow
import enum Gemstone.GemChartSection
import Localization

extension GemAssetMarketRow {
    var title: String {
        switch self {
        case .marketCap: Localized.Asset.marketCap
        case .fullyDilutedValuation: Localized.Info.FullyDilutedValuation.title
        case .tradingVolume: Localized.Asset.tradingVolume
        case .contract: Localized.Asset.contract
        case .circulatingSupply: Localized.Asset.circulatingSupply
        case .totalSupply: Localized.Asset.totalSupply
        case .maxSupply: Localized.Info.MaxSupply.title
        case .allTimeHigh: Localized.Asset.allTimeHigh
        case .allTimeLow: Localized.Asset.allTimeLow
        }
    }
}

extension GemChartSection {
    var title: String? {
        switch self {
        case .priceAlerts: Localized.Settings.PriceAlerts.title
        case .setPriceAlert: Localized.PriceAlerts.SetAlert.title
        case .links: Localized.Social.links
        case .market: nil
        }
    }
}
