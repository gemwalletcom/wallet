// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation
import Gemstone
import Primitives

public struct SignerInput {
    public let type: TransferDataType
    public let asset: Primitives.Asset
    public let value: BigInt
    public let fee: Fee
    public let useMaxAmount: Bool
    public let memo: String?
    public let senderAddress: String
    public let destinationAddress: String

    public let metadata: GemTransactionLoadMetadata

    public init(
        type: TransferDataType,
        asset: Primitives.Asset,
        value: BigInt,
        fee: Fee,
        isMaxAmount: Bool,
        memo: String?,
        senderAddress: String,
        destinationAddress: String,
        metadata: GemTransactionLoadMetadata = .none,
    ) {
        self.type = type
        self.asset = asset
        self.value = value
        self.fee = fee
        useMaxAmount = isMaxAmount
        self.memo = memo
        self.senderAddress = senderAddress
        self.destinationAddress = destinationAddress
        self.metadata = metadata
    }
}
