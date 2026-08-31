// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation
import Gemstone
import Primitives

public extension GemGasPriceType {
    func map() throws -> GasPriceType {
        switch self {
        case let .regular(gasPrice):
            try GasPriceType.regular(gasPrice: BigInt.from(string: gasPrice))
        case let .eip1559(gasPrice, priorityFee):
            try GasPriceType.eip1559(gasPrice: BigInt.from(string: gasPrice), priorityFee: BigInt.from(string: priorityFee))
        case let .solana(gasPrice, priorityFee, unitPrice):
            try GasPriceType.solana(gasPrice: BigInt.from(string: gasPrice), priorityFee: BigInt.from(string: priorityFee), unitPrice: BigInt.from(string: unitPrice))
        }
    }
}

public extension GasPriceType {
    func map() -> GemGasPriceType {
        switch self {
        case let .regular(gasPrice):
            .regular(gasPrice: gasPrice.description)
        case let .eip1559(gasPrice, priorityFee):
            .eip1559(gasPrice: gasPrice.description, priorityFee: priorityFee.description)
        case let .solana(gasPrice, priorityFee, unitPrice):
            .solana(gasPrice: gasPrice.description, priorityFee: priorityFee.description, unitPrice: unitPrice.description)
        }
    }
}
