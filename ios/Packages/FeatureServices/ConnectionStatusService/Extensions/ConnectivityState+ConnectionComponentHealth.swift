// Copyright (c). Gem Wallet. All rights reserved.

import ConnectivityService
import Primitives

extension ConnectivityState {
    var health: ConnectionComponentHealth {
        switch self {
        case .unknown:
            ConnectionComponentHealth(isHealthy: true, metadata: .none)
        case let .satisfied(path):
            ConnectionComponentHealth(
                isHealthy: true,
                metadata: .internet(InternetConnectionMetadata(
                    isExpensive: path.isExpensive,
                    isConstrained: path.isConstrained,
                    isVpn: path.isVPN,
                )),
            )
        case .unsatisfied:
            ConnectionComponentHealth(isHealthy: false, metadata: .none)
        }
    }
}
