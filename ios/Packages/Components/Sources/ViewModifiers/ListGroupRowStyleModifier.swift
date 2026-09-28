// Copyright (c). Gem Wallet. All rights reserved.

import Style
import SwiftUI

struct ListGroupRowStyleModifier: ViewModifier {
    let color: Color

    func body(content: Content) -> some View {
        content
            .cleanListRow(listRowBackground: color)
    }
}

public extension View {
    func listGroupRowStyle(color: Color = Colors.grayBackground) -> some View {
        modifier(ListGroupRowStyleModifier(color: color))
    }
}
