// Copyright (c). Gem Wallet. All rights reserved.

import Primitives

enum ConnectivityStatusState {
    case result(Latency)
    case error
    case none
}
