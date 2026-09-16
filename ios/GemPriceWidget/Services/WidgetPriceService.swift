// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import class Gemstone.GemApiClient
import enum Gemstone.GemImage
import struct Gemstone.GemWidgetCoin
import protocol Gemstone.GemWidgetServiceProtocol
import class Gemstone.GemWidgetService
import enum Gemstone.GemWidgetSize
import NativeProviderService
import Primitives
import Style
import SwiftUI
import WidgetKit

struct WidgetPriceService {
    private let service: any GemWidgetServiceProtocol
    private let preferences = SharedPreferences()

    init(service: any GemWidgetServiceProtocol = GemWidgetService(api: GemApiClient(provider: NativeProvider()))) {
        self.service = service
    }

    var refreshInterval: TimeInterval {
        TimeInterval(service.refreshIntervalSeconds())
    }

    func topCoinPrices(widgetFamily: WidgetFamily = .systemMedium) async -> PriceWidgetEntry {
        let currency = preferences.currency
        do {
            let coins = try await service.coins(size: widgetFamily.widgetSize, currency: currency)
            return await PriceWidgetEntry(
                date: Date(),
                coinPrices: coinPrices(coins: coins),
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
    private func coinPrices(coins: [GemWidgetCoin]) async -> [CoinPrice] {
        await withTaskGroup(of: (Int, CoinPrice).self) { group in
            for (index, coin) in coins.enumerated() {
                group.addTask {
                    await (index, CoinPrice(coin: coin, image: Self.image(for: coin.assetId)))
                }
            }
            return await group.reduce(into: []) { $0.append($1) }.sorted { $0.0 < $1.0 }.map(\.1)
        }
    }

    private static func image(for assetId: String) async -> Image? {
        guard let assetId = try? AssetId(id: assetId) else { return nil }
        switch assetId.type {
        case .native: return Images.name(assetId.chain.rawValue)
        case .token: return await remoteImage(url: URL(string: GemImage.asset(assetId: assetId.identifier).url()))
        }
    }

    private static func remoteImage(url: URL?) async -> Image? {
        guard let url else { return nil }
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

extension WidgetFamily {
    var widgetSize: GemWidgetSize {
        switch self {
        case .systemSmall: .small
        case .systemLarge, .systemExtraLarge: .large
        default: .medium
        }
    }
}
