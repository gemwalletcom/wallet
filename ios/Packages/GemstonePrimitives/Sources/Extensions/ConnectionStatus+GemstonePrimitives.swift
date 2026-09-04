// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import func Gemstone.connectionStatus
import Primitives

public extension [Primitives.ConnectionComponent] {
    var connectionStatus: Primitives.ConnectionStatus {
        Gemstone.connectionStatus(unhealthyComponents: map { $0.map() }).map()
    }
}
