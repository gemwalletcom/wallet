// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.GemConfirmData
import struct Gemstone.GemConfirmLoad
import struct Gemstone.GemConfirmMetadata
import struct Gemstone.GemFeeAsset
import enum Gemstone.GemTransactionInputType
import Components
import Foundation
import Primitives

struct ConfirmTransferState {
    var feeAsset: Asset
    var load: GemConfirmLoad?
    var simulation: ConfirmSimulationState
    var transaction: StateViewType<ConfirmTransferInput>
    var confirmation: ConfirmationPhase = .idle

    var metadata: GemConfirmMetadata? { load?.metadata }
    var feeAssets: [GemFeeAsset] { load?.feeAssets ?? [] }
    var confirmData: GemConfirmData? { load?.preload?.confirmData }
    var addressName: AddressName? { load?.addressName.map(AddressName.init(core:)) }
}

extension ConfirmTransferState {
    init(inputType: GemTransactionInputType, simulation: ConfirmSimulationState) {
        self.init(
            feeAsset: inputType.feeAsset().map(),
            load: nil,
            simulation: simulation,
            transaction: .loading,
        )
    }

    init(_ load: GemConfirmLoad) throws {
        let input = try load.preload.map {
            ConfirmTransferInput(
                confirmData: $0.confirmData,
                fee: $0.confirmData.fee,
                transferAmount: $0.amount.map(),
                feeAsset: load.feeAsset.map(),
            )
        }
        self.init(
            feeAsset: load.feeAsset.map(),
            load: load,
            simulation: try ConfirmSimulationState(load.simulation),
            transaction: input.map { .data($0) } ?? .loading,
        )
    }

    var transactionError: ConfirmTransferError? {
        if case let .error(error) = transaction { return ConfirmTransferError(error: error) }
        if case let .failure(error)? = transaction.value?.transferAmount { return ConfirmTransferError(error: error) }
        return nil
    }
}
