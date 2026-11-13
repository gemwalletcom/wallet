// Copyright (c). Gem Wallet. All rights reserved.

import Network

struct NWPathMapper {
    func state(from path: NWPath) -> ConnectivityState {
        switch path.status {
        case .satisfied:
            .satisfied
        case .unsatisfied:
            .unsatisfied
        case .requiresConnection:
            .unknown
        @unknown default:
            .unknown
        }
    }
}
