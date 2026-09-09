// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import struct Gemstone.GemBannerContent
import protocol Gemstone.GemWalletHomeServiceProtocol
import struct Gemstone.GemWalletHomeViewState
import Primitives

public extension GemWalletHomeServiceProtocol {
    func viewState(wallet: Wallet, balances: [AssetFiatValue], perpetual: PerpetualBalance?, banners: [Banner], isWalletEmpty: Bool) -> GemWalletHomeViewState {
        viewState(
            wallet: wallet.map(),
            balances: balances.map { $0.map() },
            perpetual: perpetual?.map(),
            banners: banners.map { $0.map() },
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
        bannerContent(event: banner.event.map(), asset: banner.asset?.map())
    }

    func applyAction(_ action: BannerAction) async throws {
        try await applyBannerAction(key: action.banner.gemKey, action: action.type.gemAction)
    }
}
