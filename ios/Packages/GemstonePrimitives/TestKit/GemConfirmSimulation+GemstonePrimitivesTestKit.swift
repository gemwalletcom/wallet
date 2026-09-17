// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.GemConfirmSimulation
import struct Gemstone.GemSimulationBalanceChange
import struct Gemstone.GemSimulationValue
import struct Gemstone.SimulationPayloadField

public extension GemConfirmSimulation {
    static func mock(
        primaryFields: [SimulationPayloadField] = [],
        header: GemSimulationValue? = nil,
        balanceChanges: [GemSimulationBalanceChange] = [],
        hasCriticalWarning: Bool = false,
    ) -> GemConfirmSimulation {
        GemConfirmSimulation(
            primaryFields: primaryFields,
            secondaryFields: [],
            header: header,
            balanceChanges: balanceChanges,
            hasCriticalWarning: hasCriticalWarning,
        )
    }
}
