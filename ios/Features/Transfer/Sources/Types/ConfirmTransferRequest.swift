// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import PaymentService
import Primitives

public struct ConfirmTransferRequest: Sendable {
    public let wallet: Wallet
    public let data: TransferData
    public let simulation: SimulationResult?
    public let delegate: StringResultAction?

    public init(
        wallet: Wallet,
        data: TransferData,
        simulation: SimulationResult? = nil,
        delegate: StringResultAction? = nil,
    ) {
        self.wallet = wallet
        self.data = data
        self.simulation = simulation
        self.delegate = delegate
    }
}
