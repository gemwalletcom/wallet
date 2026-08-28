// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives

public struct PerpetualNodeService: NodeURLProvidable {
    private let nodeProvider: any NodeURLProvidable

    public init(nodeProvider: any NodeURLProvidable) {
        self.nodeProvider = nodeProvider
    }

    public func node(for chain: Chain) -> URL {
        let url = nodeProvider.node(for: chain).toWebSocketURL()
        return url.lastPathComponent.lowercased() == "ws" ? url : url.appending(path: "ws")
    }
}
