// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Gemstone
import Primitives

public extension [Primitives.ConnectionComponent] {
    var connectionStatus: Primitives.ConnectionStatus {
        guard
            let components = try? map({ $0.json() }),
            let status = try? Primitives.ConnectionStatus(Gemstone.connectionStatus(unhealthyComponents: components))
        else {
            return isEmpty ? .online : .noService
        }
        return status
    }
}
