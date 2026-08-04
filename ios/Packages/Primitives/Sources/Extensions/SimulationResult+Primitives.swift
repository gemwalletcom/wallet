// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

public extension SimulationResult {
    static let empty = SimulationResult(
        warnings: [],
        balanceChanges: [],
        payload: [],
        header: .none,
    )
}
