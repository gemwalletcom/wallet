// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

extension TransactionState: Identifiable {
    public var id: String {
        rawValue
    }
}

public extension TransactionState {
    var isCompleted: Bool {
        switch self {
        case .confirmed, .reverted, .failed: true
        case .pending, .inTransit: false
        }
    }

    init(id: String) throws {
        if let state = TransactionState(rawValue: id) {
            self = state
        } else {
            throw AnyError("invalid state: \(id)")
        }
    }
}
