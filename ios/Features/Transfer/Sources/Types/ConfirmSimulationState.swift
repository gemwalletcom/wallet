// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.GemConfirmSimulation
import struct Gemstone.GemConfirmSimulationState
import enum Gemstone.GemListRow
import struct Gemstone.GemSimulationBalanceChange
import struct Gemstone.SimulationResult
import func Gemstone.simulationWarningRows
import GemstonePrimitives
import Primitives
import PrimitivesComponents

struct ConfirmSimulationState {
    let warnings: [GemListRow]
    let payload: SimulationPayloadModel
    let balanceChanges: [GemSimulationBalanceChange]

    init(
        warnings: [GemListRow],
        payload: SimulationPayloadModel,
        balanceChanges: [GemSimulationBalanceChange],
    ) {
        self.warnings = warnings
        self.payload = payload
        self.balanceChanges = balanceChanges
    }

    init(result: SimulationResult?) {
        self.init(
            warnings: simulationWarningRows(warnings: result?.warnings ?? []),
            payload: SimulationPayloadModel(primaryFields: [], secondaryFields: []),
            balanceChanges: [],
        )
    }

    init(_ state: GemConfirmSimulationState) {
        let details = state.simulation
        let payload = SimulationPayloadModel(
            primaryFields: details?.primaryFields ?? [],
            secondaryFields: details?.secondaryFields ?? [],
        )
        self.init(
            warnings: state.warnings,
            payload: payload,
            balanceChanges: details?.balanceChanges ?? [],
        )
    }
}
