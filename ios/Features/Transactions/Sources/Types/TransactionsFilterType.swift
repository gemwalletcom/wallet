// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import enum Gemstone.GemTransactionFilter
import Primitives

public enum TransactionsFilterType {
    case allTypes
    case type(name: GemTransactionFilter)
    case types(selected: [GemTransactionFilter])

    public init(selectedTypes: [GemTransactionFilter]) {
        switch selectedTypes.count {
        case 0: self = .allTypes
        case 1: self = .type(name: selectedTypes[0])
        default: self = .types(selected: selectedTypes)
        }
    }
}

extension TransactionsFilterType: Equatable {}
