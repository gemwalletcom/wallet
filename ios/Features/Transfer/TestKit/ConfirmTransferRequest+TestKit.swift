// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import struct Gemstone.GemTransferData
import GemstonePrimitivesTestKit
import Primitives
import PrimitivesTestKit
import Transfer
import WalletConnector

public extension ConfirmTransferRequest {
    static func mock(
        data: GemTransferData = .mock(),
        delegate: TransferDataCallback.ConfirmTransferDelegate? = nil,
    ) -> ConfirmTransferRequest {
        ConfirmTransferRequest(data: data, delegate: delegate)
    }
}
