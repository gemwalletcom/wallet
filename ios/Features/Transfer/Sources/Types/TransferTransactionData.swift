// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GemstonePrimitives
import Primitives

public struct TransferTransactionData: Sendable {
    public let rates: [FeeRate]
    public let transactionData: TransactionData
    public let scanResult: ScanTransaction?
    public let simulation: SimulationResult?

    public init(
        allRates: [FeeRate],
        transactionData: TransactionData,
        scanResult: ScanTransaction? = nil,
        simulation: SimulationResult? = nil,
    ) {
        rates = allRates
        self.transactionData = transactionData
        self.scanResult = scanResult
        self.simulation = simulation
    }
}
