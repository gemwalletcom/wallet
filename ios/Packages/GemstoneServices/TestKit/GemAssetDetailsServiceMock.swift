// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import typealias Gemstone.Asset
import typealias Gemstone.AssetId
import typealias Gemstone.BannerEvent
import typealias Gemstone.Deeplink
import protocol Gemstone.GemAssetDetailsServiceProtocol
import struct Gemstone.GemAssetDetails
import struct Gemstone.GemAssetDetailsInput
import struct Gemstone.GemAssetDetailsState
import struct Gemstone.GemAssetRefreshFailure
import enum Gemstone.GemBannerAction
import struct Gemstone.GemBannerContent
import struct Gemstone.GemBannerKey
import struct Gemstone.BlockExplorerLink
import struct Gemstone.GemSwapPairSuggestion
import enum Gemstone.WalletType
import Primitives

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

    public func details(input: GemAssetDetailsInput) -> GemAssetDetails {
        GemAssetDetails(
            state: GemAssetDetailsState(
                isViewOnly: input.walletType == .view,
                headerActions: input.walletType == .view ? .watchOnly : .buttons(buttons: []),
                showsBanners: input.walletType != .view,
                showsManage: !input.metadata.isBalanceEnabled,
                showsResources: false,
                showsPriceAlerts: false,
                priceAlertsCount: 0,
                priceAlertEnabled: false,
                showsEarn: false,
                emptyTransactionsAction: nil,
            ),
            explorerName: "Explorer",
            addressLink: input.ownerAddress.map { Gemstone.BlockExplorerLink(name: "Explorer", link: "https://gemwallet.com/\($0)") },
            tokenLink: .none,
            verificationStatus: .none,
            networkDestination: .none,
            shareUrl: "https://gemwallet.com",
            swapPair: assetPair ?? GemSwapPairSuggestion(payAssetId: input.asset.id, receiveAssetId: nil),
        )
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
