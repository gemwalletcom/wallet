// Copyright (c). Gem Wallet. All rights reserved.

import Blockchain
import ChainService
import Foundation
import protocol Gemstone.GemStakeServiceProtocol
import GemstonePrimitives
import Primitives
import Store

public struct StakeService: StakeServiceable {
    private let store: StakeStore
    private let service: any GemStakeServiceProtocol

    public init(
        store: StakeStore,
        service: any GemStakeServiceProtocol,
    ) {
        self.store = store
        self.service = service
    }

    public func stakeApr(assetId: AssetId) throws -> Double? {
        try store.getStakeApr(assetId: assetId)
    }

    public func update(walletId: WalletId, chain: Chain, address: String) async throws {
        let apr = try stakeApr(assetId: chain.assetId) ?? 0
        try await service.sync(walletId: walletId.id, chain: chain.rawValue, address: address, apr: apr)
    }

    public func clearDelegations() throws {
        try store.clearDelegations()
    }

    public func clearValidators() throws {
        try store.clearValidators()
    }
}
