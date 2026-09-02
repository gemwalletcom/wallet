// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.GemConfirmMetadata
import struct Gemstone.GemFeeAsset
import Components
import Foundation
import Primitives

struct ConfirmTransferState {
    var simulation: ConfirmSimulationState
    var metadata: GemConfirmMetadata?
    var feeRates: [FeeRate] = []
    var feeAsset: Asset
    var feeAssets: [GemFeeAsset] = []
    var transaction: StateViewType<ConfirmTransferInput>
    var confirmation: ConfirmationPhase = .idle
}

extension ConfirmTransferState {
    static func loaded(_ data: ConfirmTransferData) -> ConfirmTransferState {
        ConfirmTransferState(
            simulation: data.simulation,
            metadata: data.preload.metadata,
            feeRates: data.preload.feeRates,
            feeAsset: data.preload.input.feeAsset,
            feeAssets: data.feeAssets,
            transaction: .data(data.preload.input),
        )
    }

    var transactionError: ConfirmTransferError? {
        if case let .error(error) = transaction { return ConfirmTransferError(error: error) }
        if case let .failure(error)? = transaction.value?.transferAmount { return .amount(error) }
        return nil
    }
}
