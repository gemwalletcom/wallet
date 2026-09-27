// Copyright (c). Gem Wallet. All rights reserved.

import Components
import struct Gemstone.GemNftEntry
import Style
import SwiftUI

public struct CollectionsPreviewView: View {
    private let entries: [GemNftEntry]

    public init(entries: [GemNftEntry]) {
        self.entries = entries
    }

    public var body: some View {
        ForEach(entries, id: \.row.id) { entry in
            NavigationLink(value: entry.destination) {
                ListItemView(model: entry.posterModel.listItem)
            }
        }
    }
}
