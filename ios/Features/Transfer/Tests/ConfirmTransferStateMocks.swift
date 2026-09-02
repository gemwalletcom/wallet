// Copyright (c). Gem Wallet. All rights reserved.

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
        simulation: ConfirmSimulationState = .mock(),
        feeRates: [FeeRate] = [],
        feeAsset: Asset = .mock(),
    ) -> ConfirmTransferState {
        ConfirmTransferState(simulation: simulation, metadata: metadata, feeRates: feeRates, feeAsset: feeAsset, transaction: transaction)
    }
}
