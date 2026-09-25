// Copyright (c). Gem Wallet. All rights reserved.

import Formatters
import Foundation
import Primitives
import Style
import SwiftHTTPClient
import SwiftUI
import WidgetKit
import WidgetLocalization

struct WidgetPriceService {
    private let provider: Provider<WidgetAssetsTarget>
    private let preferences = SharedPreferences()

    init(provider: Provider<WidgetAssetsTarget> = Provider()) {
        self.provider = provider
    }

    var refreshInterval: TimeInterval {
        900
    }

    func topCoinPrices(widgetFamily: WidgetFamily = .systemMedium) async -> PriceWidgetEntry {
        let currency = preferences.currency
        do {
            let assetIds = Self.assetIds(for: widgetFamily)
            let assets = try await provider
                .request(WidgetAssetsTarget(assetIds: assetIds.map(\.identifier), currency: currency))
                .map(as: [AssetBasic].self)
            return PriceWidgetEntry(
                date: Date(),
                coinPrices: Self.coinPrices(assetIds: assetIds, assets: assets, currency: currency, widgetFamily: widgetFamily),
            )
        } catch {
            return PriceWidgetEntry.error(error: isNetworkError(error) ? error.localizedDescription : WidgetLocalized.Widget.empty)
        }
    }
}

// MARK: - Private

extension WidgetPriceService {
    static func assetIds(for widgetFamily: WidgetFamily) -> [AssetId] {
        let chains: [Chain] = switch widgetFamily {
        case .systemSmall: [.bitcoin]
        case .systemLarge, .systemExtraLarge: [.bitcoin, .ethereum, .solana, .xrp, .smartChain]
        default: [.bitcoin, .ethereum, .solana]
        }
        return chains.map { AssetId(chain: $0, tokenId: .none) }
    }

    static func coinPrices(assetIds: [AssetId], assets: [AssetBasic], currency: String, widgetFamily: WidgetFamily) -> [CoinPrice] {
        let byIdentifier = Dictionary(assets.map { ($0.asset.id.identifier, $0) }, uniquingKeysWith: { first, _ in first })
        return assetIds.compactMap { assetId in
            guard let basic = byIdentifier[assetId.identifier], let price = basic.price else { return .none }
            return CoinPrice(
                assetId: assetId,
                name: basic.asset.name,
                symbol: basic.asset.symbol,
                priceText: priceText(price.price, currency: currency, widgetFamily: widgetFamily),
                changeText: PercentFormatter().string(price.priceChangePercentage24h),
                changeTone: WidgetValueTone(change: price.priceChangePercentage24h),
                image: Images.name(assetId.chain.rawValue),
            )
        }
    }

    static func priceText(_ value: Double, currency: String, widgetFamily: WidgetFamily) -> String {
        let fiat = value.formatted(.currency(code: currency).precision(.fractionLength(2)))
        guard widgetFamily == .systemSmall else { return fiat }
        return AbbreviatedFormatter().string(from: value, currency: currency) ?? fiat
    }
}
