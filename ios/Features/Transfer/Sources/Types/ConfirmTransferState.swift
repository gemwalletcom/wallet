// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import Primitives

struct ConfirmTransferState: Sendable {
    var simulation: ConfirmSimulationState
    var metadata: TransferDataMetadata?
    var transaction: StateViewType<ConfirmTransferInput>
    var confirmation: ConfirmationPhase = .idle
}

extension ConfirmTransferState {
    static func loaded(_ data: ConfirmTransferData) -> ConfirmTransferState {
        ConfirmTransferState(simulation: data.simulation, metadata: data.metadata, transaction: .data(data.input))
    }

    var transactionError: Error? {
        if case let .error(error) = transaction { return error }
        if case let .failure(error)? = transaction.value?.transferAmount { return error }
        return nil
    }
}
