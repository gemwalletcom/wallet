// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Style
import SwiftUI

public struct CollectionsPreviewView: View {
    private let content: CollectionsContent

    public init(content: CollectionsContent) {
        self.content = content
    }

    public var body: some View {
        ForEach(content.items) { item in
            NavigationLink(value: item.destination) {
                ListItemView(model: item.model.listItem)
            }
        }
    }
}
