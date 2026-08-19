// Copyright (c). Gem Wallet. All rights reserved.

import BalanceService
import Foundation
import func Gemstone.walletAssetIsEnabled
import GemstonePrimitives
import Primitives

public struct WalletSetupService: Sendable {
    private let balanceService: BalanceService

    public init(balanceService: BalanceService) {
        self.balanceService = balanceService
    }

    public func setup(wallet: Wallet) throws {
        let chains = wallet.chains
        let defaultAssets = chains.flatMap(\.defaultAssets).assetIds
        let assetIds = chains.filter { AssetScore.defaultRank(chain: $0) >= 0 }.ids + defaultAssets

        let (enabledAssets, disabledAssets) = assetIds.reduce(into: ([AssetId](), [AssetId]())) { result, assetId in
            if walletAssetIsEnabled(assetId: assetId.identifier, walletType: wallet.type.map()) {
                result.0.append(assetId)
            } else {
                result.1.append(assetId)
            }
        }

        try balanceService.addAssetsBalancesIfMissing(assetIds: enabledAssets, wallet: wallet, isEnabled: true)
        try balanceService.addAssetsBalancesIfMissing(assetIds: disabledAssets, wallet: wallet, isEnabled: false)
    }
}
