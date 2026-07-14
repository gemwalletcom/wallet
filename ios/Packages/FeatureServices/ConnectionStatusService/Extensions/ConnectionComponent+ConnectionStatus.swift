// Copyright (c). Gem Wallet. All rights reserved.

import Primitives

extension ConnectionComponent {
    var failureStatus: ConnectionStatus {
        switch self {
        case .internet: .noInternet
        case .api, .nodes, .stream: .noService
        }
    }
}
