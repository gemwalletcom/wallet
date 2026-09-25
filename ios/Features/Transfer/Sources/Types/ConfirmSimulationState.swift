// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.GemConfirmSimulation
import struct Gemstone.GemConfirmSimulationState
import struct Gemstone.GemSimulationBalanceChange
import struct Gemstone.GemSimulationPayloadRow
import GemstonePrimitives
import Primitives
import PrimitivesComponents

struct ConfirmSimulationState {
    let primaryFields: [GemSimulationPayloadRow]
    let secondaryFields: [GemSimulationPayloadRow]
    let balanceChanges: [GemSimulationBalanceChange]

    init(
        primaryFields: [GemSimulationPayloadRow] = [],
        secondaryFields: [GemSimulationPayloadRow] = [],
        balanceChanges: [GemSimulationBalanceChange] = [],
    ) {
        self.primaryFields = primaryFields
        self.secondaryFields = secondaryFields
        self.balanceChanges = balanceChanges
    }

    init(_ state: GemConfirmSimulationState) {
        let details = state.simulation
        self.init(
            primaryFields: details?.primaryFields ?? [],
            secondaryFields: details?.secondaryFields ?? [],
            balanceChanges: details?.balanceChanges ?? [],
        )
    }
}
