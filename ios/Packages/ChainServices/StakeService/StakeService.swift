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
        let validatorNamesById = try await updateValidators(chain: chain, address: address)
        try await updateDelegations(walletId: walletId, chain: chain, address: address, validatorNamesById: validatorNamesById)
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
    private func updateValidators(chain: Chain, address: String) async throws -> [String: String] {
        let apr = try stakeApr(assetId: chain.assetId) ?? 0
        let service = chainServiceFactory.service(for: chain)

        async let getValidators = service.getValidators(apr: apr)
        async let getDelegationValidators = service.getDelegationValidators(address: address)

        let (validators, delegationValidators) = try await (
            getValidators,
            getDelegationValidators,
        )
        let validatorsList = await (try? assetsService.getValidators(chain: chain).toMap { $0.id }) ?? [:]

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
        return validatorsList.mapValues { $0.name }
    }

    private func updateDelegations(walletId: WalletId, chain: Chain, address: String, validatorNamesById: [String: String]) async throws {
        let delegations = try await getDelegations(chain: chain, address: address)
        let existingValidators = try store.getValidators(assetId: chain.assetId, providerType: .stake).toMap { $0.id }
        let delegationsValidatorIds = delegations.map(\.validatorId).asSet()
        let missingValidatorIds = delegationsValidatorIds.subtracting(existingValidators.keys)
        let missingValidators = missingValidatorIds.map { validatorId in
            DelegationValidator.inactive(
                chain: chain,
                id: validatorId,
                name: validatorNamesById[validatorId].flatMap { $0.isEmpty ? nil : $0 } ?? validatorId,
            )
        }
        if !missingValidators.isEmpty {
            try store.updateValidators(missingValidators)
        }

        let validators = existingValidators.merging(missingValidators.toMap { $0.id }) { current, _ in current }
        let updateDelegations = delegations.map { delegation in
            guard let validator = validators[delegation.validatorId] else {
                return delegation
            }
            guard delegation.state == .active, !validator.isActive else {
                return delegation
            }
            return delegation.with(state: .inactive)
        }

        let existingDelegationsIds = try store.getDelegations(walletId: walletId, assetId: chain.assetId, providerType: .stake).map(\.id).asSet()
        let updateDelegationsIds = updateDelegations.map(\.id).asSet()
        let deleteDelegationsIds = existingDelegationsIds.subtracting(updateDelegationsIds).asArray()

        try store.updateAndDelete(walletId: walletId, delegations: updateDelegations, deleteIds: deleteDelegationsIds)
    }

    private func getDelegations(chain: Chain, address: String) async throws -> [DelegationBase] {
        let service = chainServiceFactory.service(for: chain)
        return try await service.getStakeDelegations(address: address)
    }
}
