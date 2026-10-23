// Copyright (c). Gem Wallet. All rights reserved.

import SwiftUI

struct ListView<Item: Identifiable, Content: View, TrailingRows: View>: View {
    let items: [Item]
    @ViewBuilder let content: (Item) -> Content
    @ViewBuilder let trailingRows: () -> TrailingRows

    var body: some View {
        List {
            Group {
                ForEach(items) { item in
                    content(item)
                }
                trailingRows()
            }
            .listRowInsets(.assetListRowInsets)
            .listSectionSpacing(.compact)
        }
        .contentMargins(.top, .scene.top, for: .scrollContent)
    }
}
