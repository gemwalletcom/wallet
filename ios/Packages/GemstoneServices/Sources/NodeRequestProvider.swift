// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import protocol Gemstone.GemNodeServiceProtocol
import GemstonePrimitives
import Primitives
import WebSocketClient

public struct NodeRequestProvider: WebSocketRequestProvider {
    private let nodeService: any GemNodeServiceProtocol
    private let chain: Chain

    public init(nodeService: any GemNodeServiceProtocol, chain: Chain) {
        self.nodeService = nodeService
        self.chain = chain
    }

    public func makeRequest() -> URLRequest {
        URLRequest(url: URL(string: nodeService.websocketNodeUrl(chain: chain.rawValue)) ?? chain.defaultBaseUrl)
    }
}
