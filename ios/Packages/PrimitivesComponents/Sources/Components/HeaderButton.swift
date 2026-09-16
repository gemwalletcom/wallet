// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import enum Gemstone.GemHeaderButtonKind
import Localization
import Style
import SwiftUI

public enum HeaderButtonViewType: Identifiable {
    case button
    case menuButton(title: String? = nil, items: [ActionMenuItemType])

    public var id: String {
        switch self {
        case .button: "button"
        case let .menuButton(title, items): "\(title ?? "")_\(items.map(\.id).joined(separator: ","))"
        }
    }
}

public struct HeaderButton: Identifiable {
    public let type: GemHeaderButtonKind
    let viewType: HeaderButtonViewType
    public let isEnabled: Bool

    public init(
        type: GemHeaderButtonKind,
        viewType: HeaderButtonViewType = .button,
        isEnabled: Bool,
    ) {
        self.type = type
        self.isEnabled = isEnabled
        self.viewType = viewType
    }

    public var id: String {
        "\(type)_\(viewType.id)"
    }

    public var title: String {
        type.title
    }

    public var image: Image {
        type.image
    }
}
