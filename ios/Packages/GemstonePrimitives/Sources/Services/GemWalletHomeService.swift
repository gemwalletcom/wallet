// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import struct Gemstone.GemBannerContent
import struct Gemstone.GemPerpetualCollateral
import protocol Gemstone.GemWalletHomeServiceProtocol
import struct Gemstone.GemWalletHomeViewState
import Primitives

public extension GemWalletHomeServiceProtocol {
    func viewState(wallet: Wallet, balances: [AssetFiatValue], perpetual: GemPerpetualCollateral?, banners: [Banner]) -> GemWalletHomeViewState {
        viewState(
            wallet: wallet.toGem(),
            balances: balances.map { $0.toGem() },
            perpetual: perpetual,
            banners: banners.map { $0.toGem() },
        )
    }

    func updateBalances(assetIds: [AssetId]) async throws {
        try await updateBalances(assetIds: assetIds.ids)
    }

    func setAssetsEnabled(assetIds: [AssetId], enabled: Bool) async throws {
        try await setAssetsEnabled(assetIds: assetIds.ids, enabled: enabled)
    }
}
