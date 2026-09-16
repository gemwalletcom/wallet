// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import struct Gemstone.GemBannerContent
import protocol Gemstone.GemWalletHomeServiceProtocol
import struct Gemstone.GemWalletHomeViewState
import Primitives

public extension GemWalletHomeServiceProtocol {
    func viewState(wallet: Wallet, balances: [AssetFiatValue], perpetual: PerpetualBalance?, banners: [Banner], isWalletEmpty: Bool) -> GemWalletHomeViewState {
        viewState(
            wallet: wallet.toGem(),
            balances: balances.map { $0.toGem() },
            perpetual: perpetual?.toGem(),
            banners: banners.map { $0.toGem() },
            isWalletEmpty: isWalletEmpty,
        )
    }

    func updateBalances(assetIds: [AssetId]) async throws {
        try await updateBalances(assetIds: assetIds.ids)
    }

    func setAssetsEnabled(assetIds: [AssetId], enabled: Bool) async throws {
        try await setAssetsEnabled(assetIds: assetIds.ids, enabled: enabled)
    }

    func setAssetPinned(assetId: AssetId, pinned: Bool) async throws {
        try await setAssetPinned(assetId: assetId.identifier, pinned: pinned)
    }

    func content(for banner: Banner) -> GemBannerContent {
        bannerContent(event: banner.event.toGem(), asset: banner.asset?.toGem())
    }

    func close(_ banner: Banner) async throws {
        try await closeBanner(key: banner.gemKey)
    }
}
