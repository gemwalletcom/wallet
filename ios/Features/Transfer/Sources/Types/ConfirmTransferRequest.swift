// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GemstonePrimitives
import Primitives
import WalletConnector
import struct Gemstone.GemTransferData

public struct ConfirmTransferRequest: Sendable {
    public let data: GemTransferData
    public let simulation: SimulationResult?
    public let delegate: TransferDataCallback.ConfirmTransferDelegate?

    public init(
        data: GemTransferData,
        simulation: SimulationResult? = nil,
        delegate: TransferDataCallback.ConfirmTransferDelegate? = nil,
    ) {
        self.data = data
        self.simulation = simulation
        self.delegate = delegate
    }
}
