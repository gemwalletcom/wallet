// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives
import PrimitivesTestKit
import Transfer

public extension ConfirmTransferRequest {
    static func mock(
        wallet: Wallet = .mock(),
        data: TransferData = .mock(),
        simulation: SimulationResult? = nil,
    ) -> ConfirmTransferRequest {
        ConfirmTransferRequest(wallet: wallet, data: data, simulation: simulation)
    }
}
