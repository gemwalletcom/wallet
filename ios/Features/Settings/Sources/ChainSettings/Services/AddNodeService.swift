// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import class Gemstone.GemChainService
import class Gemstone.GemNodeService
import GemstoneServices
import Primitives

public struct AddNodeService: Sendable {
    private let nodeService: GemNodeService
    private let gatewayService: GatewayService
    private let chainService: GemChainService

    public init(
        nodeService: GemNodeService,
        gatewayService: GatewayService,
        chainService: GemChainService,
    ) {
        self.nodeService = nodeService
        self.gatewayService = gatewayService
        self.chainService = chainService
    }

    public func add(chain: Chain, url: URL) async throws {
        try await nodeService.addNode(chain: chain.rawValue, url: url.absoluteString)
    }

    public func check(chain: Chain, url: URL) async throws -> AddNodeResult {
        let status = try await gatewayService.nodeStatus(chain: chain, url: url.absoluteString)
        guard chainService.isValidNetworkId(chain: chain.rawValue, networkId: status.chainId) else {
            throw AddNodeError.invalidNetworkId
        }
        return AddNodeResult(
            url: url,
            chainID: status.chainId,
            blockNumber: status.latestBlockNumber,
            isInSync: true,
            latency: status.latency,
        )
    }
}
