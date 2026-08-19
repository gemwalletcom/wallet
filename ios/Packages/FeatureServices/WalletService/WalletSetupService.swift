// Copyright (c). Gem Wallet. All rights reserved.

import BalanceService
import Foundation
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

        let (enabledByDefault, disabledByDefault) = assetIds.reduce(into: ([AssetId](), [AssetId]())) { result, assetId in
            if AssetConfiguration.hiddenByDefault.contains(assetId) {
                result.1.append(assetId)
            } else if AssetConfiguration.enabledByDefault.contains(assetId) || (wallet.accounts.count == 1 && chains.count == 1) {
                result.0.append(assetId)
            } else {
                result.1.append(assetId)
            }
        }

        try balanceService.addAssetsBalancesIfMissing(assetIds: enabledByDefault, wallet: wallet, isEnabled: true)
        try balanceService.addAssetsBalancesIfMissing(assetIds: disabledByDefault, wallet: wallet, isEnabled: false)
    }
}
