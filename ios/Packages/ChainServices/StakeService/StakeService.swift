// Copyright (c). Gem Wallet. All rights reserved.

import Blockchain
import ChainService
import Foundation
import GemAPI
import Primitives
import Store

public struct StakeService: StakeServiceable {
    private let store: StakeStore
    private let addressStore: AddressStore
    private let chainServiceFactory: any ChainServiceFactorable
    private let assetsService: GemAPIStaticService

    public init(
        store: StakeStore,
        addressStore: AddressStore,
        chainServiceFactory: any ChainServiceFactorable,
        assetsService: GemAPIStaticService = GemAPIStaticService(),
    ) {
        self.store = store
        self.addressStore = addressStore
        self.chainServiceFactory = chainServiceFactory
        self.assetsService = assetsService
    }

    public func stakeApr(assetId: AssetId) throws -> Double? {
        try store.getStakeApr(assetId: assetId)
    }

    public func update(walletId: WalletId, chain: Chain, address: String) async throws {
        try await updateValidators(chain: chain, address: address)
        try await updateDelegations(walletId: walletId, chain: chain, address: address)
    }

    public func clearDelegations() throws {
        try store.clearDelegations()
    }

    public func clearValidators() throws {
        try store.clearValidators()
    }
}

// MARK: - Private

extension StakeService {
    private func updateValidators(chain: Chain, address: String) async throws {
        let apr = try stakeApr(assetId: chain.assetId) ?? 0
        let service = chainServiceFactory.service(for: chain)

        async let getValidators = service.getValidators(apr: apr)
        async let getDelegationValidators = service.getDelegationValidators(address: address)
        async let getValidatorsList = assetsService.getValidators(chain: chain)

        let (validators, delegationValidators, validatorsList) = try await (
            getValidators,
            getDelegationValidators,
            getValidatorsList.toMap { $0.id },
        )

        let activeValidatorIds = validators.map(\.id).asSet()
        let updateValidators = (validators + delegationValidators.filter { !activeValidatorIds.contains($0.id) }).map {
            let name = $0.name.isEmpty ? validatorsList[$0.id]?.name ?? .empty : $0.name
            return DelegationValidator(
                chain: $0.chain,
                id: $0.id,
                name: name,
                isActive: $0.isActive,
                commission: $0.commission,
                apr: $0.apr,
                providerType: .stake,
            )
        }
        try store.updateValidators(updateValidators)

        let addressNames = updateValidators.map {
            AddressName(chain: $0.chain, address: $0.id, name: $0.name, type: .validator, status: .verified)
        }
        try addressStore.addAddressNames(addressNames)
    }

    private func updateDelegations(walletId: WalletId, chain: Chain, address: String) async throws {
        let delegations = try await getDelegations(chain: chain, address: address)
        let existingDelegationsIds = try store.getDelegations(walletId: walletId, assetId: chain.assetId, providerType: .stake).map(\.id).asSet()
        let delegationsIds = delegations.map(\.id).asSet()
        let deleteDelegationsIds = existingDelegationsIds.subtracting(delegationsIds).asArray()

        let validators = try store.getValidators(assetId: chain.assetId, providerType: .stake).toMap { $0.id }
        let delegationsValidatorIds = delegations.map(\.validatorId).asSet()
        let missingValidatorIds = delegationsValidatorIds.subtracting(validators.keys)

        if !missingValidatorIds.isEmpty {
            debugLog("missingValidatorIds \(missingValidatorIds)")
        }
        let updateDelegations = delegations.compactMap { delegation -> DelegationBase? in
            guard let validator = validators[delegation.validatorId] else {
                return nil
            }
            guard delegation.state == .active, !validator.isActive else {
                return delegation
            }
            return delegation.with(state: .inactive)
        }

        try store.updateAndDelete(walletId: walletId, delegations: updateDelegations, deleteIds: deleteDelegationsIds)
    }

    private func getDelegations(chain: Chain, address: String) async throws -> [DelegationBase] {
        let service = chainServiceFactory.service(for: chain)
        return try await service.getStakeDelegations(address: address)
    }
}
