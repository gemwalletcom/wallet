// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Gemstone
import Primitives

public extension Gemstone.TransactionUpdate {
    func map() throws -> TransactionChanges {
        let changes: [Primitives.TransactionChange] = try changes.compactMap {
            switch $0 {
            case let .hashChange(old: old, new: new):
                return .hashChange(old: old, new: new)
            case let .metadata(metadata):
                guard let value = metadata.mapToAnyCodableValue() else { return nil }
                return .metadata(value)
            case let .blockNumber(number):
                return try .blockNumber(Int.from(string: number))
            case let .networkFee(fee):
                return try .networkFee(BigInt.from(string: fee))
            }
        }
        return TransactionChanges(
            state: state.map(),
            changes: changes,
        )
    }
}
