// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives

enum NodeStatusState {
    case result(NodeStatus)
    case error(error: any Error)
    case none
}
