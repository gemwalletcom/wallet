// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import typealias Gemstone.AddressName
import typealias Gemstone.AssetId
import typealias Gemstone.DelegationBase
import typealias Gemstone.DelegationValidator
import protocol Gemstone.GemStakeStore
import GemstonePrimitives
import Primitives
import Store

public final class GemstoneStakeStore: GemStakeStore, @unchecked Sendable {
    private let store: StakeStore
    private let addressStore: AddressStore

    public init(store: StakeStore, addressStore: AddressStore) {
        self.store = store
        self.addressStore = addressStore
    }

    public func getValidators(assetId: Gemstone.AssetId) async throws -> [Gemstone.DelegationValidator] {
        try store.getValidators(assetId: Primitives.AssetId(id: assetId), providerType: .stake).map { try $0.json() }
    }

    public func upsertValidators(validators: [Gemstone.DelegationValidator]) async throws {
        try store.updateValidators(validators.map { try Primitives.DelegationValidator($0) })
    }

    public func getDelegationIds(walletId: String, assetId: Gemstone.AssetId) async throws -> [String] {
        try store.getDelegations(walletId: WalletId.from(id: walletId), assetId: Primitives.AssetId(id: assetId), providerType: .stake).map(\.id)
    }

    public func updateAndDeleteDelegations(walletId: String, delegations: [Gemstone.DelegationBase], deleteIds: [String]) async throws {
        try store.updateAndDelete(
            walletId: WalletId.from(id: walletId),
            delegations: delegations.map { try Primitives.DelegationBase($0) },
            deleteIds: deleteIds,
        )
    }

    public func saveAddressNames(names: [Gemstone.AddressName]) async throws {
        try addressStore.addAddressNames(names.map { try Primitives.AddressName($0) })
    }
}
