// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

public enum SelectableListType<T: Sendable & Identifiable>: Sendable {
    case plain([T])
    case section([ListSection<T>])

    public var items: [T] {
        switch self {
        case let .plain(items):
            items
        case let .section(sections):
            sections.map(\.values).reduce([], +)
        }
    }
}
