// Copyright (c). Gem Wallet. All rights reserved.

import SwiftUI

struct ListView<Item: Identifiable, Content: View>: View {
    let items: [Item]
    let content: (Item) -> Content

    init(items: [Item], @ViewBuilder content: @escaping (Item) -> Content) {
        self.items = items
        self.content = content
    }

    var body: some View {
        List(items) { item in
            content(item)
                .listRowInsets(.assetListRowInsets)
                .listSectionSpacing(.compact)
        }
        .contentMargins(.top, .scene.top, for: .scrollContent)
    }
}
