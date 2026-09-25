// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.GemConfirmSimulation
import struct Gemstone.GemSimulationBalanceChange
import struct Gemstone.GemSimulationPayloadRow

public extension GemConfirmSimulation {
    static func mock(
        primaryFields: [GemSimulationPayloadRow] = [],
        balanceChanges: [GemSimulationBalanceChange] = [],
        hasCriticalWarning: Bool = false,
    ) -> GemConfirmSimulation {
        GemConfirmSimulation(
            primaryFields: primaryFields,
            secondaryFields: [],
            header: nil,
            balanceChanges: balanceChanges,
            hasCriticalWarning: hasCriticalWarning,
        )
    }
}
