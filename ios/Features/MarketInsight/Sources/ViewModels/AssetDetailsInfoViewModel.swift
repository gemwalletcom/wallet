// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Formatters
import Foundation
import class Gemstone.GemAddressService
import enum Gemstone.GemAssetMarketRow
import GemstonePrimitives
import InfoSheet
import Localization
import Primitives
import PrimitivesComponents
import Style

struct AssetDetailsInfoViewModel {
    private let asset: Asset
    private let currencyFormatter: CurrencyFormatter
    private let allTime: AllTimeValueViewModel

    init(asset: Asset, currency: String) {
        self.asset = asset
        currencyFormatter = CurrencyFormatter(type: .abbreviated, currencyCode: currency)
        allTime = AllTimeValueViewModel(
            priceFormatter: CurrencyFormatter(currencyCode: currency),
            percentFormatter: PercentFormatter.signed,
        )
    }

    func marketValues(_ rows: [GemAssetMarketRow]) -> [MarketValueViewModel] {
        rows.map(marketValue)
    }

    // MARK: - Private

    private func marketValue(_ row: GemAssetMarketRow) -> MarketValueViewModel {
        switch row {
        case let .marketCap(value, rank):
            MarketValueViewModel(
                title: Localized.Asset.marketCap,
                subtitle: currencyFormatter.string(value),
                titleTag: rank.map { " #\($0) " },
                titleTagStyle: rank.map { _ in TextStyle(font: .system(.body), color: Colors.grayLight, background: Colors.grayVeryLight) },
            )
        case let .fullyDilutedValuation(value):
            MarketValueViewModel(
                title: Localized.Info.FullyDilutedValuation.title,
                subtitle: currencyFormatter.string(value),
                action: .info(.fullyDilutedValuation),
            )
        case let .tradingVolume(value):
            MarketValueViewModel(title: Localized.Asset.tradingVolume, subtitle: currencyFormatter.string(value))
        case let .contract(tokenId, explorer):
            MarketValueViewModel(
                title: Localized.Asset.contract,
                subtitle: GemAddressService.shared.format(address: tokenId, chain: asset.chain),
                action: explorer.map {
                    .explorer(ExplorerContextData(copyValue: .address(value: tokenId, chain: asset.chain), explorerLink: $0.map()))
                } ?? .none,
            )
        case let .circulatingSupply(value):
            MarketValueViewModel(title: Localized.Asset.circulatingSupply, subtitle: supply(value), action: .info(.circulatingSupply))
        case let .totalSupply(value):
            MarketValueViewModel(title: Localized.Asset.totalSupply, subtitle: supply(value), action: .info(.totalSupply))
        case let .maxSupply(value):
            MarketValueViewModel(title: Localized.Info.MaxSupply.title, subtitle: supply(value), action: .info(.maxSupply))
        case let .allTimeHigh(value):
            allTimeValue(Localized.Asset.allTimeHigh, chartValue: value.map())
        case let .allTimeLow(value):
            allTimeValue(Localized.Asset.allTimeLow, chartValue: value.map())
        }
    }

    private func allTimeValue(_ title: String, chartValue: ChartValuePercentage) -> MarketValueViewModel {
        let item = allTime.model(title: title, chartValue: chartValue)
        return MarketValueViewModel(
            title: title,
            titleExtra: item.titleExtra,
            subtitle: item.subtitle ?? "",
            subtitleExtra: item.subtitleExtra,
            subtitleExtraStyle: item.subtitleStyleExtra,
        )
    }

    private func supply(_ value: Double) -> String {
        let formatted = AbbreviatedFormatter().string(from: value) ?? NumericFormatter().string(value)
        return "\(formatted) \(asset.symbol)"
    }
}
