// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Primitives
import PrimitivesComponents
@testable import Transfer

extension ConfirmSimulationState {
    static func mock(
        warnings: [SimulationWarning] = [],
        headerData: AssetValueHeaderData? = nil,
    ) -> ConfirmSimulationState {
        ConfirmSimulationState(
            warnings: warnings,
            payload: SimulationPayloadModel(chain: .ethereum, primaryFields: [], secondaryFields: []),
            headerData: headerData,
            balanceChanges: [],
        )
    }
}

extension ConfirmTransferState {
    static func mock(
        transaction: StateViewType<ConfirmTransferInput> = .loading,
        metadata: TransferDataMetadata? = nil,
        simulation: ConfirmSimulationState = .mock(),
        feeRates: [FeeRate] = [],
    ) -> ConfirmTransferState {
        ConfirmTransferState(simulation: simulation, metadata: metadata, feeRates: feeRates, transaction: transaction)
    }
}
