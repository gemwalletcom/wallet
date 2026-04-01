// Copyright (c). Gem Wallet. All rights reserved.

import ChainService
import Foundation
import Primitives
import PrimitivesTestKit

public extension ChainServiceFactory {
    static func mock(
        nodeProvider: any NodeURLFetchable = MockNodeURLFetchable(),
    ) -> ChainServiceFactory {
        ChainServiceFactory(nodeProvider: nodeProvider)
    }
}
