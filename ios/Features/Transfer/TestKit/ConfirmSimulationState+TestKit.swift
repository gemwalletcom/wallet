// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.GemListRow
@testable import Transfer

extension ConfirmSimulationState {
    static func mock(
        warnings: [GemListRow] = [],
    ) -> ConfirmSimulationState {
        ConfirmSimulationState(
            warnings: warnings,
            primaryFields: [],
            secondaryFields: [],
            balanceChanges: [],
        )
    }
}
