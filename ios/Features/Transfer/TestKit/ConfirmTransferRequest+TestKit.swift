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
        wallet: Wallet = .mock(),
        data: GemTransferData = .mock(),
        simulation: SimulationResult? = nil,
        delegate: TransferDataCallback.ConfirmTransferDelegate? = nil,
    ) -> ConfirmTransferRequest {
        ConfirmTransferRequest(wallet: wallet, data: data, simulation: simulation, delegate: delegate)
    }
}
