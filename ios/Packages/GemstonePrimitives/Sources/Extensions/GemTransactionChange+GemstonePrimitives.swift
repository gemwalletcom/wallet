// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Gemstone
import Primitives

public extension Gemstone.TransactionChange {
    func map() throws -> Primitives.TransactionChange {
        switch self {
        case let .hashChange(old, new):
            return .hashChange(old: old, new: new)
        case let .metadata(metadata):
            guard let value = metadata.mapToAnyCodableValue() else {
                throw AnyError("unsupported transaction metadata")
            }
            return .metadata(value)
        case let .blockNumber(number):
            return try .blockNumber(Int.from(string: number))
        case let .networkFee(fee):
            return try .networkFee(BigInt.from(string: fee))
        }
    }
}
