// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.GemConfirmData
import enum Gemstone.GemConfirmError
import enum Gemstone.GemConfirmFeeSelection
import struct Gemstone.GemConfirmLoad
import struct Gemstone.GemConfirmMetadata
import struct Gemstone.GemConfirmPreload
import struct Gemstone.GemConfirmScreen
import struct Gemstone.GemFeeAsset
import struct Gemstone.GemFeeRateRows
import struct Gemstone.GemTransactionLoadFee
import struct Gemstone.GemTransferData
import Components
import Foundation
import Primitives
import PrimitivesComponents

struct ConfirmTransferState {
    var feeAsset: Asset
    var load: GemConfirmLoad?
    var simulation: ConfirmSimulationState
    var screen: GemConfirmScreen

    var metadata: GemConfirmMetadata? { load?.metadata }
    var feeAssets: [GemFeeAsset] { load?.feeAssets ?? [] }
    var confirmData: GemConfirmData? { load?.preload?.confirmData }
    var addressName: AddressName? { load?.addressName.map { $0.map() } }
}

extension ConfirmTransferState {
    init(transfer: GemTransferData, simulation: ConfirmSimulationState, screen: GemConfirmScreen) {
        self.init(
            feeAsset: transfer.feeAsset().map(),
            load: nil,
            simulation: simulation,
            screen: screen,
        )
    }

    init(_ load: GemConfirmLoad, screen: GemConfirmScreen) throws {
        self.init(
            feeAsset: load.feeAsset.map(),
            load: load,
            simulation: try ConfirmSimulationState(load.simulation),
            screen: screen,
        )
    }

    var preload: GemConfirmPreload? {
        load?.preload
    }

    var transferAmount: TransferAmountValidation? {
        preload?.amount.map()
    }

    var fee: GemTransactionLoadFee? {
        preload?.confirmData.fee
    }

    func feeRateRows(selection: GemConfirmFeeSelection) -> GemFeeRateRows? {
        confirmData?.feeRateRows(selection: selection, feeAsset: feeAsset.map())
    }

    var transactionError: ConfirmTransferError? {
        if let failure = screen.failure, failure.stage == .load { return ConfirmTransferError(error: failure.error) }
        if case let .failure(error)? = transferAmount { return ConfirmTransferError(error: error) }
        return nil
    }
}

extension Error {
    var confirmError: GemConfirmError {
        self as? GemConfirmError ?? .Load(msg: localizedDescription)
    }
}
