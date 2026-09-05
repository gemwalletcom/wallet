// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import typealias Gemstone.Asset
import typealias Gemstone.AssetId
import struct Gemstone.AssetMetaData
import typealias Gemstone.BannerEvent
import typealias Gemstone.Chain
import typealias Gemstone.Deeplink
import struct Gemstone.GemAssetBalance
import protocol Gemstone.GemAssetDetailsServiceProtocol
import struct Gemstone.GemAssetDetailsState
import enum Gemstone.GemAssetNetworkDestination
import struct Gemstone.GemAssetRefreshFailure
import enum Gemstone.GemBannerAction
import struct Gemstone.GemBannerContent
import struct Gemstone.GemBannerKey
import struct Gemstone.BlockExplorerLink
import struct Gemstone.GemSwapPairSuggestion
import enum Gemstone.VerificationStatus
import enum Gemstone.WalletType
import GemstonePrimitives
import Primitives
import PrimitivesTestKit

public final class GemAssetDetailsServiceMock: GemAssetDetailsServiceProtocol, @unchecked Sendable {
    private let assetPair: GemSwapPairSuggestion?

    public init(assetPair: GemSwapPairSuggestion? = nil) {
        self.assetPair = assetPair
    }

    public func refresh(assetId _: AssetId) async -> [GemAssetRefreshFailure] {
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

    public func state(walletType: WalletType, chain _: Chain, metadata: AssetMetaData, balance _: GemAssetBalance, bannerEvents _: [BannerEvent], hasPrice _: Bool, priceAlertsCount _: UInt32) -> GemAssetDetailsState {
        GemAssetDetailsState(
            isViewOnly: walletType == .view,
            headerButtons: [],
            showsBanners: walletType != .view,
            showsManage: !metadata.isBalanceEnabled,
            showsResources: false,
            showsPriceAlerts: false,
            showsEarn: false,
            emptyTransactionsAction: nil,
        )
    }

    public func swapPair(assetId: AssetId, hasBalance _: Bool) -> GemSwapPairSuggestion {
        assetPair ?? GemSwapPairSuggestion(payAssetId: assetId, receiveAssetId: nil)
    }

    public func explorerName(chain _: Chain) -> String {
        "Explorer"
    }

    public func addressUrl(chain _: Chain, address: String) -> Gemstone.BlockExplorerLink {
        Gemstone.BlockExplorerLink(name: "Explorer", link: "https://gemwallet.com/\(address)")
    }

    public func tokenUrl(chain _: Chain, address _: String) -> Gemstone.BlockExplorerLink? {
        .none
    }

    public func setPriceAlert(assetId _: AssetId, enabled _: Bool) async throws {}

    public func syncPriceAlerts(assetId _: AssetId?) async throws {}

    public func deeplinkUrl(deeplink _: Deeplink) -> String {
        "https://gemwallet.com"
    }

    public func deeplinkGemUrl(deeplink _: Deeplink) -> String {
        "https://gemwallet.com"
    }
}
