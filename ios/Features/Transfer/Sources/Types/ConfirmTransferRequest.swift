// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives
import WalletConnector

public struct ConfirmTransferRequest: Sendable {
    public let wallet: Wallet
    public let data: TransferData
    public let simulation: SimulationResult?
    public let delegate: TransferDataCallback.ConfirmTransferDelegate?

    public init(
        wallet: Wallet,
        data: TransferData,
        simulation: SimulationResult? = nil,
        delegate: TransferDataCallback.ConfirmTransferDelegate? = nil,
    ) {
        self.wallet = wallet
        self.data = data
        self.simulation = simulation
        self.delegate = delegate
    }
}
