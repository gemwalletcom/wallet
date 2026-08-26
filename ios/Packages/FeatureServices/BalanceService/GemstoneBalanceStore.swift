// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import protocol Gemstone.GemBalanceStore
import struct Gemstone.GemBalanceUpdate
import enum Gemstone.GemBalanceUpdateType
import struct Gemstone.GemBalanceValue
import GemstonePrimitives
import Primitives
import Store

public final class GemstoneBalanceStore: GemBalanceStore, @unchecked Sendable {
    private let store: BalanceStore

    public init(store: BalanceStore) {
        self.store = store
    }

    public func updateBalances(walletId: String, updates: [GemBalanceUpdate]) async throws {
        let walletId = try WalletId.from(id: walletId)
        let balances = try updates.map { update in
            try UpdateBalance(
                assetId: AssetId(id: update.assetId),
                type: updateType(update.updateType),
                updatedAt: .now,
                isActive: update.isActive,
            )
        }
        try store.addBalance(assetIds: balances.map(\.assetId), isEnabled: false, for: walletId)
        try store.updateBalances(balances, for: walletId)
    }

    private func updateType(_ type: GemBalanceUpdateType) throws -> UpdateBalanceType {
        switch type {
        case let .coin(available, reserved, pendingUnconfirmed):
            .coin(UpdateCoinBalance(available: value(available), reserved: value(reserved), pendingUnconfirmed: value(pendingUnconfirmed)))
        case let .token(available):
            .token(UpdateTokenBalance(available: value(available)))
        case let .stake(staked, pending, rewards, locked, frozen, metadata):
            try .stake(UpdateStakeBalance(
                staked: value(staked),
                pending: value(pending),
                frozen: value(frozen),
                locked: value(locked),
                rewards: value(rewards),
                metadata: metadata.map { try BalanceMetadata($0) },
            ))
        case let .earn(balance):
            .earn(UpdateEarnBalance(balance: value(balance)))
        }
    }

    private func value(_ value: GemBalanceValue) -> UpdateBalanceValue {
        UpdateBalanceValue(value: value.value, amount: value.amount)
    }
}
