// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import protocol Gemstone.GemBalanceServiceProtocol
import Primitives

public extension GemBalanceServiceProtocol {
    func setAssetsEnabled(wallet: Wallet, assetIds: [AssetId], enabled: Bool) async throws {
        try await setAssetsEnabled(walletId: wallet.id.id, assetIds: assetIds.ids, enabled: enabled)
    }

    func setAssetPinned(wallet: Wallet, assetId: AssetId, pinned: Bool) async throws {
        try await setAssetPinned(walletId: wallet.id.id, assetId: assetId.identifier, pinned: pinned)
    }
}
