// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives

public protocol PaymentAssetsProvidable: Sendable {
    func assetsData(walletId: WalletId, assetIds: [AssetId]) -> [AssetData]
}
