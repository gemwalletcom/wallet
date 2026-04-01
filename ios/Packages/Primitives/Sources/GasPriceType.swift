// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation

public enum GasPriceType: Equatable, Sendable {
    case regular(gasPrice: BigInt)
    case eip1559(gasPrice: BigInt, priorityFee: BigInt)
    case solana(gasPrice: BigInt, priorityFee: BigInt, unitPrice: BigInt)

    public var gasPrice: BigInt {
        switch self {
        case let .regular(gasPrice): gasPrice
        case let .eip1559(gasPrice, _): gasPrice
        case let .solana(gasPrice, _, _): gasPrice
        }
    }

    public var priorityFee: BigInt {
        switch self {
        case .regular: .zero
        case let .eip1559(_, priorityFee): priorityFee
        case let .solana(_, priorityFee, _): priorityFee
        }
    }

    public var unitPrice: BigInt {
        switch self {
        case .regular: .zero
        case .eip1559: .zero
        case let .solana(_, _, unitPrice): unitPrice
        }
    }

    public var totalFee: BigInt {
        gasPrice + priorityFee
    }
}

extension GasPriceType: Hashable {}
