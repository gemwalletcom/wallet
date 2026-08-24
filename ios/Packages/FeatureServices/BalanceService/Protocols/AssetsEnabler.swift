// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives

public protocol AssetsEnabler: Sendable {
    func enableAssets(wallet: Wallet, assetIds: [AssetId], enabled: Bool) async throws
    func pinAsset(wallet: Wallet, assetId: AssetId, pinned: Bool) async throws
}
