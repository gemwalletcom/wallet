// Copyright (c). Gem Wallet. All rights reserved.

import Primitives

extension ConnectionStatus {
    var severity: Int {
        switch self {
        case .online: 0
        case .noService: 1
        case .noInternet: 2
        }
    }
}
