// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Gemstone
import Primitives

public extension [ConnectionComponent] {
    var connectionStatus: ConnectionStatus {
        Gemstone.connectionStatus(unhealthyComponents: map { $0.map() }).map()
    }
}

extension ConnectionComponent {
    func map() -> GemConnectionComponent {
        switch self {
        case .internet: .internet
        case .api: .api
        case .nodes: .nodes
        case .stream: .stream
        }
    }
}

extension GemConnectionStatus {
    func map() -> ConnectionStatus {
        switch self {
        case .online: .online
        case .noInternet: .noInternet
        case .noService: .noService
        }
    }
}
