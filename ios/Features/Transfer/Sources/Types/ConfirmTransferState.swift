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

    var transactionError: ConfirmTransferError? {
        if case let .error(error) = transaction { return ConfirmTransferError(error: error) }
        if case let .failure(error)? = transaction.value?.transferAmount { return .amount(error) }
        return nil
    }
}
