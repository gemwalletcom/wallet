// Copyright (c). Gem Wallet. All rights reserved.

import Primitives

enum ConnectivityStatusState: Sendable {
    case result(Latency)
    case error
    case none
}
