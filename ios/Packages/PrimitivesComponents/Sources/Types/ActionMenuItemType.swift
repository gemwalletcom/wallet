// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives
import SwiftUI

public enum ActionMenuItemType: Identifiable {
    case button(
        title: String,
        systemImage: String? = nil,
        role: ButtonRole? = nil,
        action: VoidAction,
    )

    public var id: String {
        switch self {
        case let .button(title, _, _, _): "button-\(title)"
        }
    }
}
