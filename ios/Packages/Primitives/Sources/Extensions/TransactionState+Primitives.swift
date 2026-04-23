// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

extension TransactionState: Identifiable {
    public var id: String {
        rawValue
    }

    public var isPending: Bool {
        self == .pending
    }

    public var isCompleted: Bool {
        switch self {
        case .pending, .inTransit: false
        case .confirmed, .failed, .reverted: true
        }
    }
}

public extension TransactionState {
    init(id: String) throws {
        if let state = TransactionState(rawValue: id) {
            self = state
        } else {
            throw AnyError("invalid state: \(id)")
        }
    }
}
