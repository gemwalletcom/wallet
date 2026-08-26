// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation
import Gemstone
import Primitives

public struct TransactionInput: Sendable {
    public let type: TransferDataType
    public let asset: Primitives.Asset
    public let senderAddress: String
    public let destinationAddress: String
    public let value: BigInt
    public let balance: BigInt
    public let gasPrice: GasPriceType
    public let memo: String?
    public let metadata: GemTransactionLoadMetadata

    public init(
        type: TransferDataType,
        asset: Primitives.Asset,
        senderAddress: String,
        destinationAddress: String,
        value: BigInt,
        balance: BigInt,
        gasPrice: GasPriceType,
        memo: String?,
        metadata: GemTransactionLoadMetadata,
    ) {
        self.type = type
        self.asset = asset
        self.senderAddress = senderAddress
        self.destinationAddress = destinationAddress
        self.value = value
        self.balance = balance
        self.gasPrice = gasPrice
        self.memo = memo
        self.metadata = metadata
    }
}

public extension TransactionInput {
    var feeInput: FeeInput {
        FeeInput(
            type: type,
            senderAddress: senderAddress,
            destinationAddress: destinationAddress,
            value: value,
            balance: balance,
            gasPrice: gasPrice,
            memo: memo,
        )
    }
}
