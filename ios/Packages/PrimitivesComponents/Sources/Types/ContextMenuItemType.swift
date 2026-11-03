// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.GemRowMenuItem
import Primitives
import SwiftUI

public enum ContextMenuItemType {
    case copy(title: String? = nil, value: String, expirationTime: TimeInterval? = nil, onCopy: StringAction = nil)
    case pin(isPinned: Bool, onPin: VoidAction)
    case delete(VoidAction)
    case url(title: String, onOpen: VoidAction)
    case custom(
        title: String,
        systemImage: String? = nil,
        role: ButtonRole? = nil,
        action: VoidAction,
    )

    var group: ContextMenuGroup {
        switch self {
        case .copy, .url: .view
        case .pin: .actions
        case .delete: .destructive
        case let .custom(_, _, role, _): role == .destructive ? .destructive : .actions
        }
    }
}

enum ContextMenuGroup {
    case view
    case actions
    case destructive
}

public extension [GemRowMenuItem] {
    func contextMenuItems(onOpen: @escaping (URL) -> Void) -> [ContextMenuItemType] {
        map { item in
            switch item {
            case let .copy(copy): .copy(value: copy.value)
            case let .open(title, url): .url(title: title.text, onOpen: { URL(string: url).map(onOpen) })
            }
        }
    }
}
