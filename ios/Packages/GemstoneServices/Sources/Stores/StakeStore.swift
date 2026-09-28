// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import typealias Gemstone.AssetId
import struct Gemstone.DelegationBase
import struct Gemstone.DelegationValidator
import protocol Gemstone.GemStakeStore
import typealias Gemstone.StakeProviderType
import GemstonePrimitives
import Primitives
import Store

public final class GemstoneStakeStore: GemStakeStore, @unchecked Sendable {
    private let store: StakeStore

    public init(store: StakeStore) {
        self.store = store
    }

    public func getApr(assetId: Gemstone.AssetId, providerType: Gemstone.StakeProviderType) async throws -> Double? {
        switch providerType.toPrimitives() {
        case .stake: return try store.getStakeApr(assetId: assetId)
        case .earn: return try store.getEarnApr(assetId: assetId)
        }
    }

    public func getValidators(assetId: Gemstone.AssetId, providerType: Gemstone.StakeProviderType) async throws -> [Gemstone.DelegationValidator] {
        try store.getValidators(assetId: assetId, providerType: providerType.toPrimitives()).map { $0.toGem() }
    }

    public func saveValidators(validators: [Gemstone.DelegationValidator]) async throws {
        try store.updateValidators(validators.map { $0.toPrimitives() })
    }

    public func deactivateValidators(assetId: Gemstone.AssetId, validatorIds: [String]) async throws {
        try store.deactivateValidators(assetId: assetId, validatorIds: validatorIds)
    }

    public func getDelegationIds(walletId: WalletId, assetId: Gemstone.AssetId, providerType: Gemstone.StakeProviderType) async throws -> [String] {
        try store.getDelegationIds(walletId: walletId, assetId: assetId, providerType: providerType.toPrimitives())
    }

    public func updateDelegations(walletId: WalletId, delegations: [Gemstone.DelegationBase], deleteIds: [String]) async throws {
        try store.updateAndDelete(
            walletId: walletId,
            delegations: delegations.map { $0.toPrimitives() },
            deleteIds: deleteIds,
        )
    }
}
