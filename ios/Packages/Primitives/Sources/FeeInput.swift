// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation

public struct FeeInput: Sendable {
    public let type: TransferDataType
    public let senderAddress: String
    public let destinationAddress: String
    public let value: BigInt
    public let balance: BigInt
    public let gasPrice: GasPriceType
    public let memo: String?

    public init(
        type: TransferDataType,
        senderAddress: String,
        destinationAddress: String,
        value: BigInt,
        balance: BigInt,
        gasPrice: GasPriceType,
        memo: String?,
    ) {
        self.type = type
        self.senderAddress = senderAddress
        self.destinationAddress = destinationAddress
        self.value = value
        self.balance = balance
        self.gasPrice = gasPrice
        self.memo = memo
    }

    public var isMaxAmount: Bool {
        balance > 0 && balance == value
    }

    public var chain: Chain {
        type.chain
    }

    public var gasLimit: BigInt? {
        switch type {
        case let .swap(_, _, data): data.data.gasLimit.flatMap { BigInt(stringLiteral: $0) }
        case let .generic(_, _, extra): extra.gasLimit
        default: .none
        }
    }
}
