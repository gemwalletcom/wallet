// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import SwiftUI

public extension UIColor {
    var color: Color {
        Color(self)
    }

    class func dynamicColor(_ light: Color, dark: Color?) -> Color {
        UIColor { traitCollection in
            switch traitCollection.userInterfaceStyle {
            case .dark:
                dark?.uiColor ?? light.uiColor
            default:
                light.uiColor
            }
        }.color
    }
}
