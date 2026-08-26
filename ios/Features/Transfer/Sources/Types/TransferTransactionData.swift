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

    func withFeeAsset(_ feeAssetId: AssetId) -> TransferTransactionData {
        let fee = transactionData.fee
        return TransferTransactionData(
            allRates: rates,
            transactionData: TransactionData(
                fee: Fee(
                    fee: fee.fee,
                    gasPriceType: fee.gasPriceType,
                    gasLimit: fee.gasLimit,
                    options: fee.options,
                    feeAssetId: feeAssetId,
                ),
                metadata: transactionData.metadata,
            ),
            scanResult: scanResult,
            simulation: simulation,
        )
    }
}
