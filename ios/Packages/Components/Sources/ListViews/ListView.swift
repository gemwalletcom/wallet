// Copyright (c). Gem Wallet. All rights reserved.

import SwiftUI

struct ListView<Item: Identifiable, Content: View>: View {
    let items: [Item]
    @ViewBuilder let content: (Item) -> Content

    var body: some View {
        List(items) { item in
            content(item)
                .listRowInsets(.assetListRowInsets)
                .listSectionSpacing(.compact)
        }
        .contentMargins(.top, .scene.top, for: .scrollContent)
    }
}
