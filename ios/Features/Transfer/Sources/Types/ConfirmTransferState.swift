// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import enum Gemstone.GemConfirmError
import struct Gemstone.GemConfirmFee
import struct Gemstone.GemConfirmLoad
import struct Gemstone.GemConfirmScreen
import struct Gemstone.GemTransferData
import Primitives
import PrimitivesComponents

struct ConfirmTransferState {
    var transfer: GemTransferData
    var feeAsset: Asset
    var load: GemConfirmLoad?
    var screen: GemConfirmScreen

    var fee: GemConfirmFee? { load?.fee }
}

extension ConfirmTransferState {
    init(transfer: GemTransferData, screen: GemConfirmScreen) {
        self.init(
            transfer: transfer,
            feeAsset: transfer.feeAsset().toPrimitives(),
            load: nil,
            screen: screen,
        )
    }

    init(_ load: GemConfirmLoad, screen: GemConfirmScreen) {
        self.init(
            transfer: load.transfer,
            feeAsset: load.feeAsset.toPrimitives(),
            load: load,
            screen: screen,
        )
    }

    var loadError: GemConfirmError? {
        guard let failure = screen.failure, failure.stage == .load else { return nil }
        return failure.error
    }
}
