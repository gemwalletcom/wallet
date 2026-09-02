// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import struct Gemstone.GemBannerContent
import protocol Gemstone.GemWalletHomeServiceProtocol
import Primitives

public extension GemWalletHomeServiceProtocol {
    func refresh(wallet: Wallet, assetIds: [AssetId]) async throws {
        try await refresh(walletId: wallet.id.id, assetIds: assetIds.ids)
    }

    func setAssetsEnabled(wallet: Wallet, assetIds: [AssetId], enabled: Bool) async throws {
        try await setAssetsEnabled(walletId: wallet.id.id, assetIds: assetIds.ids, enabled: enabled)
    }

    func setAssetPinned(wallet: Wallet, assetId: AssetId, pinned: Bool) async throws {
        try await setAssetPinned(walletId: wallet.id.id, assetId: assetId.identifier, pinned: pinned)
    }

    func content(for banner: Banner) -> GemBannerContent {
        bannerContent(event: banner.event.json(), asset: banner.asset?.map())
    }

    func applyAction(_ action: BannerAction) async throws {
        try await applyBannerAction(key: action.banner.gemKey, action: action.type.gemAction)
    }
}
