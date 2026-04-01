// Copyright (c). Gem Wallet. All rights reserved.

import SwiftUI

struct ToolbarDismissItemModifier: ViewModifier {
    let type: ToolbarDismissItem.ButtonType
    let placement: ToolbarItemPlacement

    init(
        type: ToolbarDismissItem.ButtonType,
        placement: ToolbarItemPlacement,
    ) {
        self.type = type
        self.placement = placement
    }

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
