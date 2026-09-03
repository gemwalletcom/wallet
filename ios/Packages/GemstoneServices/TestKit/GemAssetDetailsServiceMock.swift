// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import typealias Gemstone.Asset
import typealias Gemstone.AssetFull
import typealias Gemstone.AssetId
import typealias Gemstone.BannerEvent
import typealias Gemstone.Chain
import typealias Gemstone.Deeplink
import protocol Gemstone.GemAssetDetailsServiceProtocol
import enum Gemstone.GemAssetNetworkDestination
import struct Gemstone.GemAssetRefreshFailure
import enum Gemstone.GemBannerAction
import struct Gemstone.GemBannerContent
import struct Gemstone.GemBannerKey
import struct Gemstone.GemBlockExplorerLink
import class Gemstone.GemDeeplinkService
import class Gemstone.GemSimulationFormatter
import struct Gemstone.GemSwapPairSuggestion
import enum Gemstone.VerificationStatus
import GemstonePrimitives
import Primitives
import PrimitivesTestKit

public final class GemAssetDetailsServiceMock: GemAssetDetailsServiceProtocol, @unchecked Sendable {
    private let assetPair: GemSwapPairSuggestion?
    private let deeplinks = GemDeeplinkService()

    public init(assetPair: GemSwapPairSuggestion? = nil) {
        self.assetPair = assetPair
    }

    public func refresh(assetId _: AssetId) async -> [GemAssetRefreshFailure] {
        []
    }

    public func syncAsset(assetId _: AssetId) async throws -> AssetFull {
        Primitives.AssetFull.mock().json()
    }

    public func syncMissingAssets(assetIds _: [AssetId]) async throws -> [AssetId] {
        []
    }

    public func syncTransactions(assetId _: AssetId?) async throws {}

    public func updateBalances(assetIds _: [AssetId]) async throws {}

    public func setAssetPinned(assetId _: AssetId, pinned _: Bool) async throws {}

    public func setAssetsEnabled(assetIds _: [AssetId], enabled _: Bool) async throws {}

    public func addPrices(assetIds _: [AssetId]) async throws {}

    public func bannerContent(event _: BannerEvent, asset _: Asset?) -> GemBannerContent {
        GemBannerContent(icon: .none, title: .none, description: .none, link: .none)
    }

    public func applyBannerAction(key _: GemBannerKey, action _: GemBannerAction) async throws {}

    public func networkDestination(assetId _: AssetId) -> GemAssetNetworkDestination? {
        .none
    }

    public func verificationStatus(asset _: Asset, rank _: Int32) -> Gemstone.VerificationStatus? {
        .none
    }

    public func swapPair(assetId: AssetId, hasBalance _: Bool) -> GemSwapPairSuggestion {
        assetPair ?? GemSwapPairSuggestion(payAssetId: assetId, receiveAssetId: nil)
    }

    public func explorerName(chain _: Chain) -> String {
        "Explorer"
    }

    public func addressUrl(chain _: Chain, address: String) -> GemBlockExplorerLink {
        GemBlockExplorerLink(name: "Explorer", link: "https://gemwallet.com/\(address)")
    }

    public func tokenUrl(chain _: Chain, address _: String) -> GemBlockExplorerLink? {
        .none
    }

    public func setPriceAlert(assetId _: AssetId, enabled _: Bool) async throws {}

    public func syncPriceAlerts(assetId _: AssetId?) async throws {}

    public func deeplinkUrl(deeplink: Deeplink) -> String {
        deeplinks.buildUrl(deeplink: deeplink)
    }

    public func deeplinkGemUrl(deeplink: Deeplink) -> String {
        deeplinks.buildGemUrl(deeplink: deeplink)
    }
}
