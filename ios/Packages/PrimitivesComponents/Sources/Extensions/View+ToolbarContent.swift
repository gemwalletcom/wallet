// Copyright (c). Gem Wallet. All rights reserved.

import SwiftUI

public extension View {
    func toolbarContent(@ToolbarContentBuilder _ content: @escaping () -> some ToolbarContent) -> some View {
        toolbar(content: content)
    }
}
