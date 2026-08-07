// Copyright (c). Gem Wallet. All rights reserved.

import Gemstone
import Primitives

public extension Gemstone.GemServiceEndpoint {
    func map() -> Primitives.ServiceEndpoint {
        Primitives.ServiceEndpoint(
            type: endpointType.map(),
            host: host,
            url: url,
            flag: flag,
        )
    }
}

public extension Gemstone.GemServiceEndpointType {
    func map() -> Primitives.ServiceEndpointType {
        switch self {
        case .api: .api
        case .gemNode: .gemNode
        }
    }
}
