// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import typealias Gemstone.Asset
import typealias Gemstone.AssetId
import typealias Gemstone.AssetFull
import typealias Gemstone.BannerEvent
import typealias Gemstone.Chain
import typealias Gemstone.Currency
import typealias Gemstone.Deeplink
import protocol Gemstone.GemAssetDetailsServiceProtocol
import struct Gemstone.GemBannerContent
import enum Gemstone.GemBannerAction
import struct Gemstone.GemBannerKey
import struct Gemstone.GemAssetRefreshFailure
import struct Gemstone.GemBlockExplorerLink
import class Gemstone.GemDeeplinkService
import class Gemstone.GemSimulationFormatter
import struct Gemstone.GemSwapPairSuggestion
import typealias Gemstone.PriceAlert
import typealias Gemstone.WalletId
import GemstonePrimitives
import Primitives
import PrimitivesTestKit

public final class GemAssetDetailsServiceMock: GemAssetDetailsServiceProtocol, @unchecked Sendable {
    private let assetPair: GemSwapPairSuggestion?
    private let deeplinks = GemDeeplinkService()

    public init(assetPair: GemSwapPairSuggestion? = nil) {
        self.assetPair = assetPair
    }

    public func refresh(walletId: WalletId, assetId: AssetId, currency: Currency) async -> [GemAssetRefreshFailure] { [] }

    public func syncAsset(assetId: AssetId, currency: Currency) async throws -> AssetFull {
        Primitives.AssetFull.mock().json()
    }

    public func syncMissingAssets(assetIds: [AssetId]) async throws -> [AssetId] { [] }

    public func syncTransactions(walletId: WalletId, assetId: AssetId?) async throws {}

    public func updateBalances(walletId: WalletId, assetIds: [AssetId]) async throws {}

    public func setAssetPinned(walletId: WalletId, assetId: AssetId, pinned: Bool) async throws {}

    public func setAssetsEnabled(walletId: WalletId, assetIds: [AssetId], enabled: Bool) async throws {}

    public func addPrices(assetIds: [AssetId]) async throws {}

    public func bannerContent(event: BannerEvent, asset: Asset?) -> GemBannerContent {
        GemBannerContent(icon: .none, title: .none, description: .none)
    }

    public func applyBannerAction(key: GemBannerKey, action: GemBannerAction) async throws {}

    public func swapPair(assetId: AssetId, hasBalance: Bool) -> GemSwapPairSuggestion {
        assetPair ?? GemSwapPairSuggestion(payAssetId: assetId, receiveAssetId: nil)
    }

    public func addressUrl(chain: Chain, address: String) -> GemBlockExplorerLink {
        GemBlockExplorerLink(name: "Explorer", link: "https://gemwallet.com/\(address)")
    }

    public func tokenUrl(chain: Chain, address: String) -> GemBlockExplorerLink? { .none }

    public func enablePriceAlert(alert: PriceAlert) async throws {}

    public func deletePriceAlerts(alerts: [PriceAlert]) async throws {}

    public func syncPriceAlerts(assetId: AssetId?) async throws {}

    public func deeplinkUrl(deeplink: Deeplink) -> String {
        deeplinks.buildUrl(deeplink: deeplink)
    }

    public func deeplinkGemUrl(deeplink: Deeplink) -> String {
        deeplinks.buildGemUrl(deeplink: deeplink)
    }
}
