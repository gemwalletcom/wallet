// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.GemListRow
import PrimitivesComponents
@testable import Transfer

extension ConfirmSimulationState {
    static func mock(
        warnings: [GemListRow] = [],
    ) -> ConfirmSimulationState {
        ConfirmSimulationState(
            warnings: warnings,
            payload: SimulationPayloadModel(primaryFields: [], secondaryFields: []),
            balanceChanges: [],
        )
    }
}
