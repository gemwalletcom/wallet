// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
@testable import Primitives

public extension NodeURLProvidable {
    static func mock() -> NodeURLProvidable {
        NodeURLProviderMock()
    }
}

public struct NodeURLProviderMock: NodeURLProvidable {
    public func node(for _: Chain) -> URL {
        URL(string: "https://mock-node.example.com")!
    }

    public init() {}
}
