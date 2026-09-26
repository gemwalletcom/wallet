// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.GemConfirmLoad
import struct Gemstone.GemConfirmScreen
import struct Gemstone.GemTransferData
import GemstonePrimitivesTestKit
import Primitives
import PrimitivesTestKit
@testable import Transfer

extension ConfirmTransferState {
    static func mock(
        transfer: GemTransferData = .mock(),
        load: GemConfirmLoad? = nil,
        feeAsset: Asset = .mock(),
        screen: GemConfirmScreen = .mock(),
    ) -> ConfirmTransferState {
        ConfirmTransferState(transfer: load?.transfer ?? transfer, feeAsset: feeAsset, load: load, screen: screen)
    }
}
