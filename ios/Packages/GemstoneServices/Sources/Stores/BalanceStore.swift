// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import struct Gemstone.AssetBalance
import typealias Gemstone.AssetId
import struct Gemstone.GemAssetConfiguration
import struct Gemstone.GemBalanceRecord
import protocol Gemstone.GemBalanceStore
import struct Gemstone.GemBalanceValue
import GemstonePrimitives
import Primitives
import Store

public final class GemstoneBalanceStore: GemBalanceStore, @unchecked Sendable {
    private let store: BalanceStore

    public init(store: BalanceStore) {
        self.store = store
    }

    public func getAvailableBalances(walletId: WalletId, assetIds: [Gemstone.AssetId]) throws -> [Gemstone.AssetBalance] {
        try store.getBalances(walletId: walletId, assetIds: assetIds)
            .map { $0.toGem() }
    }

    public func getBalanceAssetIds(walletId: WalletId, assetIds: [Gemstone.AssetId]) throws -> [Gemstone.AssetId] {
        try store.getBalanceAssetIds(walletId: walletId, assetIds: assetIds)
    }

    public func addBalances(walletId: WalletId, assetIds: [Gemstone.AssetId], enabled: Bool) async throws {
        try store.addBalance(assetIds: assetIds, isEnabled: enabled, for: walletId)
    }

    public func updateBalances(walletId: WalletId, balances: [GemBalanceRecord]) async throws {
        let updates = balances.map { balance in
            UpdateBalance(
                assetId: balance.assetId,
                available: value(balance.available),
                frozen: value(balance.frozen),
                locked: value(balance.locked),
                staked: value(balance.staked),
                pending: value(balance.pending),
                pendingUnconfirmed: value(balance.pendingUnconfirmed),
                rewards: value(balance.rewards),
                reserved: value(balance.reserved),
                withdrawable: value(balance.withdrawable),
                earn: value(balance.earn),
                metadata: balance.metadata.map { $0.toPrimitives() },
                updatedAt: .now,
                isActive: balance.isActive,
            )
        }
        try store.updateBalances(updates, for: walletId)
    }

    public func getEnabledAssetIds(walletId: WalletId) async throws -> [Gemstone.AssetId] {
        try store.getEnabledAssetIds(walletId: walletId)
    }

    public func setAssetConfiguration(walletId: WalletId, assetIds: [Gemstone.AssetId], configuration: GemAssetConfiguration) async throws {
        try store.setConfiguration(
            walletId: walletId,
            assetIds: assetIds,
            configuration: AssetConfiguration(isEnabled: configuration.isEnabled, isPinned: configuration.isPinned),
        )
    }

    private func value(_ value: GemBalanceValue) -> UpdateBalanceValue {
        UpdateBalanceValue(value: value.value.description, amount: value.amount)
    }
}
