// Copyright (c). Gem Wallet. All rights reserved.

import Gemstone
import Primitives

public extension Gemstone.TransactionUpdate {
    func map() throws -> TransactionChanges {
        try TransactionChanges(state: state.map(), changes: changes.map { try $0.map() })
    }
}
