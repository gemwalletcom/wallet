// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.GemConfirmFailure
import struct Gemstone.GemConfirmLoad
import enum Gemstone.GemConfirmPhase
import struct Gemstone.GemConfirmPreload
import struct Gemstone.GemConfirmScreen
import Components
import Primitives
import PrimitivesComponents
import struct Gemstone.GemSimulationWarningRow
import struct Gemstone.SimulationResult
@testable import Transfer

extension ConfirmSimulationState {
    static func mock(
        result: SimulationResult? = nil,
        warnings: [GemSimulationWarningRow] = [],
        headerData: AssetValueHeaderData? = nil,
    ) -> ConfirmSimulationState {
        ConfirmSimulationState(
            result: result,
            warnings: warnings,
            hasCriticalWarning: warnings.contains { $0.severity == .critical },
            payload: SimulationPayloadModel(chain: .ethereum, primaryFields: [], secondaryFields: []),
            headerData: headerData,
            balanceChanges: [],
        )
    }
}

extension ConfirmTransferState {
    static func mock(
        load: GemConfirmLoad? = nil,
        simulation: ConfirmSimulationState = .mock(),
        feeAsset: Asset = .mock(),
        screen: GemConfirmScreen = .mock(),
    ) -> ConfirmTransferState {
        ConfirmTransferState(feeAsset: feeAsset, load: load, simulation: simulation, screen: screen)
    }
}

extension GemConfirmScreen {
    static func mock(
        phase: GemConfirmPhase = .loading,
        amountFailed: Bool = false,
        hasCriticalWarning: Bool = false,
        failure: GemConfirmFailure? = nil,
    ) -> GemConfirmScreen {
        GemConfirmScreen(phase: phase, amountFailed: amountFailed, hasCriticalWarning: hasCriticalWarning, failure: failure)
    }
}
