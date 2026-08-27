// Copyright (c). Gem Wallet. All rights reserved.
import Foundation
import SwiftUI

public extension Color {
    static func random() -> Color {
        Color(
            red: .random(in: 0 ... 1),
            green: .random(in: 0 ... 1),
            blue: .random(in: 0 ... 1),
            opacity: 1,
        )
    }
}
