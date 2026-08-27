// Copyright (c). Gem Wallet. All rights reserved.

import GemstoneServices
import BigInt
import Foundation
import Primitives

public final class ChainServiceMock: ChainServiceable, @unchecked Sendable {
    // Injected data
    public var chainID: String?
    public var latestBlock: BigInt = .zero
    public var validators: [DelegationValidator] = []
    public var delegations: [DelegationBase] = []
    public var inSync: Bool = true
    public var tokenData: [String: Asset] = [:]
    public var tokenDataError: (any Error)?
    public var nodeStatus: NodeStatus = .init(chainId: "1", latestBlockNumber: .zero, latency: .from(duration: 1000))

    public init() {}
}

public extension ChainServiceMock {
    func getChainID() async throws -> String {
        chainID ?? ""
    }

    func getLatestBlock() async throws -> BigInt {
        latestBlock
    }

    func getValidators(apr _: Double) async throws -> [DelegationValidator] {
        validators
    }

    func getStakeDelegations(address _: String) async throws -> [DelegationBase] {
        delegations
    }

    func getInSync() async throws -> Bool {
        inSync
    }

    func getTokenData(tokenId: String) async throws -> Asset {
        if let asset = tokenData[tokenId] {
            return asset
        }
        if let tokenDataError {
            throw tokenDataError
        }
        return Asset(
            id: AssetId(chain: .ethereum, tokenId: nil),
            name: "Ethereum",
            symbol: "ETH",
            decimals: 18,
            type: .native,
        )
    }

    func getIsTokenAddress(tokenId: String) -> Bool {
        tokenData[tokenId] != nil
    }

    func getNodeStatus(url _: String) async throws -> NodeStatus {
        nodeStatus
    }
}
