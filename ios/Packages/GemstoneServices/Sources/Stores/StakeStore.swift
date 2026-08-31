// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import typealias Gemstone.AssetId
import typealias Gemstone.DelegationBase
import typealias Gemstone.DelegationValidator
import typealias Gemstone.StakeProviderType
import protocol Gemstone.GemStakeStore
import GemstonePrimitives
import Primitives
import Store

public final class GemstoneStakeStore: GemStakeStore, @unchecked Sendable {
    private let store: StakeStore

    public init(store: StakeStore) {
        self.store = store
    }

    public func getApr(assetId: Gemstone.AssetId, providerType: Gemstone.StakeProviderType) async throws -> Double? {
        let assetId = try Primitives.AssetId(id: assetId)
        switch try Primitives.StakeProviderType(providerType) {
        case .stake: return try store.getStakeApr(assetId: assetId)
        case .earn: return try store.getEarnApr(assetId: assetId)
        }
    }

    public func getValidators(assetId: Gemstone.AssetId, providerType: Gemstone.StakeProviderType) async throws -> [Gemstone.DelegationValidator] {
        try store.getValidators(assetId: Primitives.AssetId(id: assetId), providerType: Primitives.StakeProviderType(providerType)).map { $0.json() }
    }

    public func saveValidators(validators: [Gemstone.DelegationValidator]) async throws {
        try store.updateValidators(validators.map { try Primitives.DelegationValidator($0) })
    }

    public func deactivateValidators(assetId: Gemstone.AssetId, validatorIds: [String]) async throws {
        try store.deactivateValidators(assetId: Primitives.AssetId(id: assetId), validatorIds: validatorIds)
    }

    public func getDelegationIds(walletId: String, assetId: Gemstone.AssetId, providerType: Gemstone.StakeProviderType) async throws -> [String] {
        try store.getDelegations(walletId: WalletId.from(id: walletId), assetId: Primitives.AssetId(id: assetId), providerType: Primitives.StakeProviderType(providerType)).map(\.id)
    }

    public func updateDelegations(walletId: String, delegations: [Gemstone.DelegationBase], deleteIds: [String]) async throws {
        try store.updateAndDelete(
            walletId: WalletId.from(id: walletId),
            delegations: delegations.map { try Primitives.DelegationBase($0) },
            deleteIds: deleteIds,
        )
    }

}
