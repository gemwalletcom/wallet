// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import class Gemstone.GemNodeService
import GemstonePrimitives
import Primitives

extension GemNodeService: @retroactive NodeURLProvidable {
    public func node(for chain: Chain) -> URL {
        URL(string: nodeUrl(chain: chain.rawValue)) ?? chain.defaultBaseUrl
    }
}
