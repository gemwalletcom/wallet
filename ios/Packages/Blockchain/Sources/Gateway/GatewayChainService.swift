// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation
import Primitives

import enum Gemstone.GemTransactionLoadMetadata

internal import GemstonePrimitives

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

    func feeRates(type: TransferDataType) async throws -> [FeeRate] {
        try await gateway.feeRates(chain: chain, input: type)
    }

    func preload(input: TransactionPreloadInput) async throws -> GemTransactionLoadMetadata {
        try await gateway.transactionPreload(chain: chain, input: input)
    }

    func load(input: TransactionInput) async throws -> TransactionData {
        try await gateway.transactionLoad(chain: chain, input: input.map())
    }

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
