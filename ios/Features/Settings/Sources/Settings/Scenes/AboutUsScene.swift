// Copyright (c). Gem Wallet. All rights reserved.

import Components
import PrimitivesComponents
import Style
import SwiftUI

public struct AboutUsScene: View {
    @State private var model: AboutUsViewModel

    public init(model: AboutUsViewModel) {
        _model = State(initialValue: model)
    }

    public var body: some View {
        ListSectionView(provider: model) { row in
            GemListRowView(row: row)
                .contextMenu(model.contextMenuItems(for: row))
        }
        .contentMargins(.top, .scene.top, for: .scrollContent)
        .listStyle(.insetGrouped)
        .listSectionSpacing(.compact)
        .navigationTitle(model.title)
        .taskOnce { Task { await model.load() } }
    }
}
