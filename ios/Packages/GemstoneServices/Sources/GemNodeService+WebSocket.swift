// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import class Gemstone.GemNodeService
import GemstonePrimitives
import Primitives

extension GemNodeService {
    public func webSocketNode(for chain: Chain) -> URL {
        URL(string: websocketNodeUrl(chain: chain.rawValue)) ?? chain.defaultBaseUrl
    }
}
