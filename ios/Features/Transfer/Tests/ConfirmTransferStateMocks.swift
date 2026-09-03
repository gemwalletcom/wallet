// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.GemConfirmData
import struct Gemstone.GemConfirmMetadata
import Components
import Primitives
import PrimitivesComponents
@testable import Transfer

extension ConfirmSimulationState {
    static func mock(
        result: SimulationResult? = nil,
        warnings: [SimulationWarning] = [],
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
        transaction: StateViewType<ConfirmTransferInput> = .loading,
        metadata: GemConfirmMetadata? = nil,
        confirmData: GemConfirmData? = nil,
        simulation: ConfirmSimulationState = .mock(),
        feeAsset: Asset = .mock(),
    ) -> ConfirmTransferState {
        ConfirmTransferState(simulation: simulation, metadata: metadata, confirmData: confirmData, feeAsset: feeAsset, transaction: transaction)
    }
}
