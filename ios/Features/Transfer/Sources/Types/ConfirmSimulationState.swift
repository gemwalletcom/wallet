// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.GemConfirmSimulation
import struct Gemstone.GemConfirmSimulationState
import struct Gemstone.GemSimulationBalanceChange
import struct Gemstone.GemSimulationValue
import GemstonePrimitives
import Primitives
import PrimitivesComponents
import struct Gemstone.GemSimulationWarningRow
import struct Gemstone.SimulationResult
import func Gemstone.simulationWarningRows

struct ConfirmSimulationState {
    let result: SimulationResult?
    let warnings: [GemSimulationWarningRow]
    let hasCriticalWarning: Bool
    let payload: SimulationPayloadModel
    let headerData: GemSimulationValue?
    let balanceChanges: [GemSimulationBalanceChange]

    init(
        result: SimulationResult?,
        warnings: [GemSimulationWarningRow],
        hasCriticalWarning: Bool,
        payload: SimulationPayloadModel,
        headerData: GemSimulationValue?,
        balanceChanges: [GemSimulationBalanceChange],
    ) {
        self.result = result
        self.warnings = warnings
        self.hasCriticalWarning = hasCriticalWarning
        self.payload = payload
        self.headerData = headerData
        self.balanceChanges = balanceChanges
    }

    init(result: SimulationResult?) {
        self.init(
            result: result,
            warnings: simulationWarningRows(warnings: result?.warnings ?? []),
            hasCriticalWarning: false,
            payload: SimulationPayloadModel(primaryFields: [], secondaryFields: []),
            headerData: nil,
            balanceChanges: [],
        )
    }

    init(_ state: GemConfirmSimulationState) {
        let details = state.simulation
        let simulation = state.result
        let payload = SimulationPayloadModel(
            primaryFields: details?.primaryFields ?? [],
            secondaryFields: details?.secondaryFields ?? [],
        )
        self.init(
            result: simulation,
            warnings: state.warnings,
            hasCriticalWarning: details?.hasCriticalWarning ?? false,
            payload: payload,
            headerData: details?.header,
            balanceChanges: details?.balanceChanges ?? [],
        )
    }
}
