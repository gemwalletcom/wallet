// Copyright (c). Gem Wallet. All rights reserved.

import SwiftUI

struct ToolbarDismissItemModifier: ViewModifier {
    let type: ToolbarDismissItem.ButtonType
    let placement: ToolbarItemPlacement

    func body(content: Content) -> some View {
        content
            .toolbar {
                ToolbarDismissItem(
                    type: type,
                    placement: placement,
                )
            }
    }
}

public extension View {
    func toolbarDismissItem(
        type: ToolbarDismissItem.ButtonType,
        placement: ToolbarItemPlacement,
    ) -> some View {
        modifier(ToolbarDismissItemModifier(type: type, placement: placement))
    }
}
