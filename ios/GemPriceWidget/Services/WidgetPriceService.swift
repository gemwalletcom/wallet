// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GemAPI
import Preferences
import Primitives
import Style
import SwiftUI
import WidgetKit

struct WidgetPriceService {
    private let assetsService: any GemAPIAssetsService
    private let preferences = SharedPreferences()

    init() {
        assetsService = GemAPIService()
    }

    func coins(_ widgetFamily: WidgetFamily) -> [AssetId] {
        switch widgetFamily {
        case .systemSmall:
            [AssetId(chain: .bitcoin, tokenId: nil)]
        default:
            [
                AssetId(chain: .bitcoin, tokenId: nil),
                AssetId(chain: .ethereum, tokenId: nil),
                AssetId(chain: .solana, tokenId: nil),
                AssetId(chain: .xrp, tokenId: nil),
                AssetId(chain: .smartChain, tokenId: nil),
            ]
        }
    }

    func fetchTopCoinPrices(widgetFamily: WidgetFamily = .systemMedium) async -> PriceWidgetEntry {
        let coins = coins(widgetFamily)
        let currency = preferences.currency

        do {
            let assets = try await assetsService.getAssets(currency: currency, assetIds: coins)

            return await PriceWidgetEntry(
                date: Date(),
                coinPrices: coinPrices(assets: assets),
                currency: currency,
                widgetFamily: widgetFamily,
            )
        } catch {
            return PriceWidgetEntry.error(error: error.localizedDescription, widgetFamily: widgetFamily)
        }
    }
}

// MARK: - Private

extension WidgetPriceService {
    private func coinPrices(assets: [AssetBasic]) async -> [CoinPrice] {
        await withTaskGroup(of: CoinPrice?.self) { group in
            for asset in assets {
                guard let price = asset.price else { continue }
                group.addTask {
                    await CoinPrice(
                        assetId: asset.asset.id,
                        name: asset.asset.name,
                        symbol: asset.asset.symbol,
                        price: price.price,
                        priceChangePercentage24h: price.priceChangePercentage24h,
                        image: Self.image(for: asset.asset.id),
                    )
                }
            }
            return await group.compactMap(\.self).reduce(into: []) { $0.append($1) }
        }
    }

    private static func image(for assetId: AssetId) async -> Image? {
        switch assetId.type {
        case .native: Images.name(assetId.chain.rawValue)
        case .token: await fetchRemoteImage(url: AssetImageFormatter.shared.getURL(for: assetId))
        }
    }

    private static func fetchRemoteImage(url: URL) async -> Image? {
        do {
            let (data, _) = try await URLSession.shared.data(from: url)
            guard let uiImage = UIImage(data: data) else {
                throw AnyError("wrong image format")
            }
            return Image(uiImage: uiImage)
        } catch {
            debugLog("WidgetPriceService: Failed to fetch image from \(url): \(error)")
            return nil
        }
    }
}
