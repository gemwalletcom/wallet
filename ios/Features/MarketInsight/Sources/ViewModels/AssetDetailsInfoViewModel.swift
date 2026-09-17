// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Formatters
import Foundation
import func Gemstone.abbreviationThreshold
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

    init(asset: Asset, currency: Currency) {
        self.asset = asset
        currencyFormatter = CurrencyFormatter(type: .abbreviated, currencyCode: currency.rawValue)
        allTime = AllTimeValueViewModel(
            priceFormatter: CurrencyFormatter(currencyCode: currency.rawValue),
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
                title: row.title,
                subtitle: currencyFormatter.string(value),
                titleTag: rank.map { " #\($0) " },
                titleTagStyle: rank.map { _ in TextStyle(font: .system(.body), color: Colors.grayLight, background: Colors.grayVeryLight) },
            )
        case let .fullyDilutedValuation(value):
            MarketValueViewModel(title: row.title, subtitle: currencyFormatter.string(value), action: .info(.fullyDilutedValuation))
        case let .tradingVolume(value):
            MarketValueViewModel(title: row.title, subtitle: currencyFormatter.string(value))
        case let .contract(tokenId, explorer):
            MarketValueViewModel(
                title: row.title,
                subtitle: GemAddressService.shared.format(address: tokenId, chain: asset.chain),
                action: explorer.map {
                    .explorer(ExplorerContextData(copyValue: .address(value: tokenId, chain: asset.chain), explorerLink: $0.toPrimitives()))
                } ?? .none,
            )
        case let .circulatingSupply(value):
            MarketValueViewModel(title: row.title, subtitle: supply(value), action: .info(.circulatingSupply))
        case let .totalSupply(value):
            MarketValueViewModel(title: row.title, subtitle: supply(value), action: .info(.totalSupply))
        case let .maxSupply(value):
            MarketValueViewModel(title: row.title, subtitle: supply(value), action: .info(.maxSupply))
        case let .allTimeHigh(value):
            allTimeValue(row.title, chartValue: value.toPrimitives())
        case let .allTimeLow(value):
            allTimeValue(row.title, chartValue: value.toPrimitives())
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
        let formatted = value >= abbreviationThreshold() ? AbbreviatedFormatter().string(from: value) ?? NumericFormatter().string(value) : NumericFormatter().string(value)
        return "\(formatted) \(asset.symbol)"
    }
}
