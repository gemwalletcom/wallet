// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import protocol Gemstone.GemNodeServiceProtocol
import GemstonePrimitives
import Primitives

public extension GemNodeServiceProtocol {
    func webSocketNode(for chain: Chain) -> URL {
        URL(string: websocketNodeUrl(chain: chain.rawValue)) ?? chain.defaultBaseUrl
    }
}
