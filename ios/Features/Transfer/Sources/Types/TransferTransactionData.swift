// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives

public struct TransferTransactionData: Sendable {
    public let rates: [FeeRate]
    public let transactionData: TransactionData
    public let scanResult: ScanTransaction?

    public init(
        allRates: [FeeRate],
        transactionData: TransactionData,
        scanResult: ScanTransaction? = nil,
    ) {
        rates = allRates
        self.transactionData = transactionData
        self.scanResult = scanResult
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
        )
    }
}
