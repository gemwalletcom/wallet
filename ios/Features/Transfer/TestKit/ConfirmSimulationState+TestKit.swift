// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.GemSimulationValue
import struct Gemstone.GemSimulationWarningRow
import PrimitivesComponents
@testable import Transfer

extension ConfirmSimulationState {
    static func mock(
        warnings: [GemSimulationWarningRow] = [],
        headerData: GemSimulationValue? = nil,
    ) -> ConfirmSimulationState {
        ConfirmSimulationState(
            result: nil,
            warnings: warnings,
            hasCriticalWarning: warnings.contains { $0.severity == .critical },
            payload: SimulationPayloadModel(primaryFields: [], secondaryFields: []),
            headerData: headerData,
            balanceChanges: [],
        )
    }
}
