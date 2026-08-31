// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

public extension [SimulationPayloadField] {
    var primaryFields: [SimulationPayloadField] {
        filter { $0.display == .primary }
    }

    var secondaryFields: [SimulationPayloadField] {
        filter { $0.display == .secondary }
    }
}
