// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import struct Gemstone.GemTransferData
import struct Gemstone.SimulationResult
import Primitives
import WalletConnector

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
