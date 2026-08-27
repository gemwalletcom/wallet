// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation
import Primitives

struct GatewayChainService {
    private let chain: Chain
    let gateway: GatewayService

    init(
        chain: Chain,
        gateway: GatewayService,
    ) {
        self.chain = chain
        self.gateway = gateway
    }
}

extension GatewayChainService: ChainServiceable {
    func getChainID() async throws -> String {
        try await gateway.chainId(chain: chain)
    }

    func getLatestBlock() async throws -> BigInt {
        try await BigInt(gateway.latestBlock(chain: chain))
    }

    func getNodeStatus(url: String) async throws -> NodeStatus {
        try await gateway.nodeStatus(chain: chain, url: url)
    }

    func getValidators(apr: Double) async throws -> [DelegationValidator] {
        try await gateway.validators(chain: chain, apy: apr)
    }

    func getDelegationValidators(address: String) async throws -> [DelegationValidator] {
        try await gateway.delegationValidators(chain: chain, address: address)
    }

    func getStakeDelegations(address: String) async throws -> [DelegationBase] {
        try await gateway.delegations(chain: chain, address: address)
    }

    func getTokenData(tokenId: String) async throws -> Asset {
        try await gateway.tokenData(chain: chain, tokenId: tokenId)
    }

    func getIsTokenAddress(tokenId: String) async throws -> Bool {
        try await gateway.isTokenAddress(chain: chain, tokenId: tokenId)
    }
}
