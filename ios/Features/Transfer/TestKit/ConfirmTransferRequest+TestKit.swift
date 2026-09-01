// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GemstonePrimitivesTestKit
import GemstonePrimitives
import Primitives
import PrimitivesTestKit
import Transfer
import WalletConnector

public extension ConfirmTransferRequest {
    static func mock(
        wallet: Wallet = .mock(),
        data: TransferData = .mock(),
        simulation: SimulationResult? = nil,
        delegate: TransferDataCallback.ConfirmTransferDelegate? = nil,
    ) -> ConfirmTransferRequest {
        ConfirmTransferRequest(wallet: wallet, data: data, simulation: simulation, delegate: delegate)
    }
}
