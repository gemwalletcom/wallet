// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import typealias Gemstone.Asset
import typealias Gemstone.AssetId
import typealias Gemstone.BannerEvent
import struct Gemstone.BlockExplorerLink
import typealias Gemstone.Deeplink
import struct Gemstone.GemAssetDetails
import struct Gemstone.GemAssetDetailsInput
import protocol Gemstone.GemAssetDetailsServiceProtocol
import struct Gemstone.GemAssetDetailsState
import struct Gemstone.GemAssetRefresh
import struct Gemstone.GemBannerContent
import struct Gemstone.GemBannerKey
import struct Gemstone.GemFormattedNumber
import struct Gemstone.GemSwapPairSuggestion
import enum Gemstone.WalletType
import GemstonePrimitivesTestKit
import Primitives

public final class GemAssetDetailsServiceMock: GemAssetDetailsServiceProtocol, @unchecked Sendable {
    private let assetPair: GemSwapPairSuggestion?

    public init(assetPair: GemSwapPairSuggestion? = nil) {
        self.assetPair = assetPair
    }

    public func refresh(assetId _: AssetId, hasTransactions _: Bool) async -> GemAssetRefresh {
        GemAssetRefresh(transactions: .data, failures: [])
    }

    public func setAssetPinned(assetId _: AssetId, pinned _: Bool) async throws {}

    public func setAssetsEnabled(assetIds _: [AssetId], enabled _: Bool) async throws {}

    public func closeBanner(key _: GemBannerKey) async throws {}

    public func details(input: GemAssetDetailsInput) -> GemAssetDetails {
        GemAssetDetails(
            state: GemAssetDetailsState(
                isViewOnly: input.wallet.walletType == .view,
                headerActions: input.wallet.walletType == .view ? .watchOnly : .buttons(buttons: []),
                showsBanners: input.wallet.walletType != .view,
                priceAlert: .disabled,
                emptyTransactionsAction: nil,
            ),
            icon: .mock(),
            banner: nil,
            balanceValue: .mock(value: 0, unit: .symbol(symbol: input.assetData.asset.symbol), display: .number(precision: .fraction(min: 2, max: 2)), notation: .signed, tone: .plain, rounding: .toNearest),
            sections: [],
            title: input.assetData.asset.name,
            fiatValue: .none,
            explorerName: "Explorer",
            addressLink: input.assetData.account.address.isEmpty ? nil : Gemstone.BlockExplorerLink(name: "Explorer", link: "https://gemwallet.com/\(input.assetData.account.address)"),
            tokenLink: .none,
            verificationStatus: .none,
            networkDestination: .none,
            shareUrl: "https://gemwallet.com",
            swapPair: assetPair ?? GemSwapPairSuggestion(payAssetId: input.assetData.asset.id, receiveAssetId: nil),
        )
    }

    public func setPriceAlert(assetId _: AssetId, enabled _: Bool) async throws {}

    public func deeplinkUrl(deeplink _: Deeplink) -> String {
        "https://gemwallet.com"
    }
}
