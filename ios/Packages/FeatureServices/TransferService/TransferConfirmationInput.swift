// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import PaymentService
import Primitives

public struct TransferConfirmationInput: Sendable {
    public let data: TransferData
    public let wallet: Wallet
    public let transactionData: TransactionData
    public let amount: TransferAmount
    public let simulation: SimulationResult?
    public let delegate: StringResultAction?

    public init(
        data: TransferData,
        wallet: Wallet,
        transactionData: TransactionData,
        amount: TransferAmount,
        simulation: SimulationResult? = nil,
        delegate: StringResultAction?,
    ) {
        self.data = data
        self.wallet = wallet
        self.transactionData = transactionData
        self.amount = amount
        self.simulation = simulation
        self.delegate = delegate
    }

}
