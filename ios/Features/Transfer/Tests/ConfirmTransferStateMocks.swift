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
            primaryFields: [],
            secondaryFields: [],
            payloadAddressNames: [:],
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
    ) -> ConfirmTransferState {
        ConfirmTransferState(simulation: simulation, metadata: metadata, transaction: transaction)
    }
}
