// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.GemListRow
import struct Gemstone.GemSimulationValue
import PrimitivesComponents
@testable import Transfer

extension ConfirmSimulationState {
    static func mock(
        warnings: [GemListRow] = [],
        headerData: GemSimulationValue? = nil,
    ) -> ConfirmSimulationState {
        ConfirmSimulationState(
            result: nil,
            warnings: warnings,
            hasCriticalWarning: false,
            payload: SimulationPayloadModel(primaryFields: [], secondaryFields: []),
            headerData: headerData,
            balanceChanges: [],
        )
    }
}
