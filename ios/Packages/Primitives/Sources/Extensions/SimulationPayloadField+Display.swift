// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

public extension SimulationPayloadField {
    var isPrimary: Bool {
        display == .primary
    }
}

public extension [SimulationPayloadField] {
    var primaryFields: [SimulationPayloadField] {
        filter(\.isPrimary)
    }

    var secondaryFields: [SimulationPayloadField] {
        filter { $0.display == .secondary }
    }
}
