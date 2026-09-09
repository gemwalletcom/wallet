// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import protocol Gemstone.GemBalanceStore
import struct Gemstone.GemAssetBalance
import struct Gemstone.GemBalanceRecord
import struct Gemstone.GemBalanceValue
import typealias Gemstone.AssetId
import GemstonePrimitives
import Primitives
import Store

public final class GemstoneBalanceStore: GemBalanceStore, @unchecked Sendable {
    private let store: BalanceStore

    public init(store: BalanceStore) {
        self.store = store
    }

    public func getAvailableBalances(walletId: String, assetIds: [Gemstone.AssetId]) throws -> [GemAssetBalance] {
        let walletId = try WalletId.from(id: walletId)
        return try assetIds.compactMap { assetId in
            let assetId = try Primitives.AssetId(id: assetId)
            return try store.getBalance(walletId: walletId, assetId: assetId).map { GemAssetBalance($0.balance, assetId: assetId, isActive: $0.isActive) }
        }
    }

    public func updateBalances(walletId: String, balances: [GemBalanceRecord]) async throws {
        let walletId = try WalletId.from(id: walletId)
        let updates = try balances.map { balance in
            try UpdateBalance(
                assetId: Primitives.AssetId(id: balance.assetId),
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
                metadata: balance.metadata.map { $0.map() },
                updatedAt: .now,
                isActive: balance.isActive,
            )
        }
        try store.updateBalances(updates, for: walletId)
    }

    public func getEnabledAssetIds(walletId: String) async throws -> [Gemstone.AssetId] {
        try store.getEnabledAssetIds(walletId: WalletId.from(id: walletId)).map(\Primitives.AssetId.identifier)
    }

    public func setAssetsEnabled(walletId: String, assetIds: [Gemstone.AssetId], enabled: Bool) async throws {
        try store.setIsEnabled(walletId: WalletId.from(id: walletId), assetIds: assetIds.map { try Primitives.AssetId(id: $0) }, value: enabled)
    }

    public func setAssetPinned(walletId: String, assetId: Gemstone.AssetId, pinned: Bool) async throws {
        try store.pinAsset(walletId: WalletId.from(id: walletId), assetId: Primitives.AssetId(id: assetId), value: pinned)
    }

    private func value(_ value: GemBalanceValue) -> UpdateBalanceValue {
        UpdateBalanceValue(value: value.value.description, amount: value.amount)
    }
}
