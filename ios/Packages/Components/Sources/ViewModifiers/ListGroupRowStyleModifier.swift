// Copyright (c). Gem Wallet. All rights reserved.

import Style
import SwiftUI

public struct ListGroupRowStyleModifier: ViewModifier {
    let color: Color

    public func body(content: Content) -> some View {
        content
            .cleanListRow(listRowBackground: color)
    }
}

public extension View {
    func listGroupRowStyle(color: Color = Colors.grayBackground) -> some View {
        modifier(ListGroupRowStyleModifier(color: color))
    }
}
