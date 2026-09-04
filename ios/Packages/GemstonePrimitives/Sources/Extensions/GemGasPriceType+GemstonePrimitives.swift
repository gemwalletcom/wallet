// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Gemstone
import Primitives

public extension GemGasPriceType {
    func map() -> GasPriceType {
        switch self {
        case let .regular(gasPrice):
            .regular(gasPrice: gasPrice)
        case let .eip1559(gasPrice, priorityFee):
            .eip1559(gasPrice: gasPrice, priorityFee: priorityFee)
        case let .solana(gasPrice, priorityFee, unitPrice):
            .solana(gasPrice: gasPrice, priorityFee: priorityFee, unitPrice: unitPrice)
        }
    }
}

public extension GasPriceType {
    func map() -> GemGasPriceType {
        switch self {
        case let .regular(gasPrice):
            .regular(gasPrice: gasPrice)
        case let .eip1559(gasPrice, priorityFee):
            .eip1559(gasPrice: gasPrice, priorityFee: priorityFee)
        case let .solana(gasPrice, priorityFee, unitPrice):
            .solana(gasPrice: gasPrice, priorityFee: priorityFee, unitPrice: unitPrice)
        }
    }
}
