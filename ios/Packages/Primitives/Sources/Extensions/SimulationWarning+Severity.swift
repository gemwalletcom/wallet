// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

public extension [SimulationWarning] {
    var hasCritical: Bool { contains { $0.severity == .critical } }
}
