// Copyright (c). Gem Wallet. All rights reserved.

import Blockchain
import Foundation
import protocol Gemstone.GemStakeServiceProtocol
import Primitives
import Store

public protocol EarnDataProvidable: Sendable {
    func getEarnData(assetId: AssetId, address: String, value: String, earnType: EarnType) async throws -> ContractCallData
}

public protocol EarnPositionsUpdatable: Sendable {
    func update(walletId: WalletId, assetId: AssetId, address: String) async throws
}

public struct EarnService: Sendable, EarnPositionsUpdatable {
    private let store: StakeStore
    private let service: any GemStakeServiceProtocol
    private let gatewayService: GatewayService

    public init(store: StakeStore, service: any GemStakeServiceProtocol, gatewayService: GatewayService) {
        self.store = store
        self.service = service
        self.gatewayService = gatewayService
    }

    public func update(walletId: WalletId, assetId: AssetId, address: String) async throws {
        let apr = try store.getEarnApr(assetId: assetId) ?? 0
        try await service.syncEarn(walletId: walletId.id, assetId: assetId.identifier, address: address, apr: apr)
    }
}

// MARK: - EarnDataProvidable

extension EarnService: EarnDataProvidable {
    public func getEarnData(assetId: AssetId, address: String, value: String, earnType: EarnType) async throws -> ContractCallData {
        try await gatewayService.getEarnData(assetId: assetId, address: address, value: value, earnType: earnType)
    }
}
