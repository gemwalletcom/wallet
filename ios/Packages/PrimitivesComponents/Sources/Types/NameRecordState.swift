// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives

public enum NameRecordState: Equatable, Hashable, Sendable {
    case none
    case loading(name: String)
    case error
    case complete(NameRecord)
}

public extension NameRecordState {
    var result: NameRecord? {
        switch self {
        case let .complete(result): result
        case .none, .loading, .error: .none
        }
    }

    var requestedName: String? {
        switch self {
        case let .loading(name): name
        case let .complete(record): record.name
        case .none, .error: .none
        }
    }
}
