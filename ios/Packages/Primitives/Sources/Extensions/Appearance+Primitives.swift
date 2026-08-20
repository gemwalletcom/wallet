// Copyright (c). Gem Wallet. All rights reserved.

import SwiftUI

extension Appearance: Identifiable {
    public var id: String { rawValue }

    public var colorScheme: ColorScheme? {
        switch self {
        case .system: .none
        case .light: .light
        case .dark: .dark
        }
    }
}
