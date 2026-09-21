// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import struct Gemstone.GemTransferData
import struct Gemstone.SimulationResult
import GemstonePrimitivesTestKit
import Primitives
import PrimitivesTestKit
import Transfer
import WalletConnector

public extension ConfirmTransferRequest {
    static func mock(
        data: GemTransferData = .mock(),
        simulation: SimulationResult? = nil,
        delegate: TransferDataCallback.ConfirmTransferDelegate? = nil,
    ) -> ConfirmTransferRequest {
        ConfirmTransferRequest(data: data, simulation: simulation, delegate: delegate)
    }
}
