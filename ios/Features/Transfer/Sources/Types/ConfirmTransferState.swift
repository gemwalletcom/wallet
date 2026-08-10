// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import Primitives

struct ConfirmTransferState {
    var simulation: ConfirmSimulationState
    var metadata: TransferDataMetadata?
    var feeRates: [FeeRate] = []
    var transaction: StateViewType<ConfirmTransferInput>
    var confirmation: ConfirmationPhase = .idle
}

extension ConfirmTransferState {
    static func loaded(_ data: ConfirmTransferData) -> ConfirmTransferState {
        ConfirmTransferState(
            simulation: data.simulation,
            metadata: data.preload.metadata,
            feeRates: data.preload.feeRates,
            transaction: .data(data.preload.input),
        )
    }

    mutating func update(_ preload: ConfirmTransferPreload) {
        metadata = preload.metadata
        feeRates = preload.feeRates
        transaction = .data(preload.input)
    }

    var transactionError: ConfirmTransferError? {
        if case let .error(error) = transaction { return ConfirmTransferError(error: error) }
        if case let .failure(error)? = transaction.value?.transferAmount { return .amount(error) }
        return nil
    }
}
