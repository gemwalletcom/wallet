// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.GemConfirmSimulation
import struct Gemstone.GemConfirmSimulationState
import enum Gemstone.GemListRow
import struct Gemstone.GemSimulationBalanceChange
import struct Gemstone.GemSimulationPayloadRow
import struct Gemstone.SimulationResult
import func Gemstone.simulationWarningRows
import GemstonePrimitives
import Primitives
import PrimitivesComponents

struct ConfirmSimulationState {
    let warnings: [GemListRow]
    let primaryFields: [GemSimulationPayloadRow]
    let secondaryFields: [GemSimulationPayloadRow]
    let balanceChanges: [GemSimulationBalanceChange]

    init(
        warnings: [GemListRow],
        primaryFields: [GemSimulationPayloadRow],
        secondaryFields: [GemSimulationPayloadRow],
        balanceChanges: [GemSimulationBalanceChange],
    ) {
        self.warnings = warnings
        self.primaryFields = primaryFields
        self.secondaryFields = secondaryFields
        self.balanceChanges = balanceChanges
    }

    init(result: SimulationResult?) {
        self.init(
            warnings: simulationWarningRows(warnings: result?.warnings ?? []),
            primaryFields: [],
            secondaryFields: [],
            balanceChanges: [],
        )
    }

    init(_ state: GemConfirmSimulationState) {
        let details = state.simulation
        self.init(
            warnings: state.warnings,
            primaryFields: details?.primaryFields ?? [],
            secondaryFields: details?.secondaryFields ?? [],
            balanceChanges: details?.balanceChanges ?? [],
        )
    }
}
