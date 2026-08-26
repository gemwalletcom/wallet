// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import protocol Gemstone.GemBalanceServiceProtocol
import Primitives
import Store

public struct BalanceService: Sendable {
    private let balanceStore: BalanceStore
    private let service: any GemBalanceServiceProtocol

    public init(
        balanceStore: BalanceStore,
        service: any GemBalanceServiceProtocol,
    ) {
        self.balanceStore = balanceStore
        self.service = service
    }
}

// MARK: - BalanceUpdater

extension BalanceService: BalanceUpdater {
    public func updateBalance(for wallet: Wallet, assetIds: [AssetId]) async {
        do {
            try await service.update(walletId: wallet.id.id, assetIds: assetIds.ids)
        } catch {
            debugLog("update balance error: \(error.localizedDescription)")
        }
    }
}

// MARK: - Balances

extension BalanceService {
    public func getBalance(walletId: WalletId, assetId: AssetId) throws -> Balance? {
        try balanceStore.getBalance(walletId: walletId, assetId: assetId)
    }

    public func addAssetsBalancesIfMissing(assetIds: [AssetId], wallet: Wallet, isEnabled: Bool) throws {
        try balanceStore.addBalance(assetIds: assetIds, isEnabled: isEnabled, for: wallet.id)
    }
}
