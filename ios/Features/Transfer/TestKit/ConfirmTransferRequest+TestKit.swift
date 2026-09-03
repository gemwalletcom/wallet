// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GemstonePrimitivesTestKit
import GemstonePrimitives
import Primitives
import PrimitivesTestKit
import Transfer
import WalletConnector
import struct Gemstone.GemTransferData

public extension ConfirmTransferRequest {
    static func mock(
        data: GemTransferData = .mock(),
        simulation: SimulationResult? = nil,
        delegate: TransferDataCallback.ConfirmTransferDelegate? = nil,
    ) -> ConfirmTransferRequest {
        ConfirmTransferRequest(data: data, simulation: simulation, delegate: delegate)
    }
}
