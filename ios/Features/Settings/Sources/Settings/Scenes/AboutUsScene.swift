// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Localization
import PrimitivesComponents
import Style
import SwiftUI

public struct AboutUsScene: View {
    @State private var model: AboutUsViewModel

    public init(model: AboutUsViewModel) {
        _model = State(initialValue: model)
    }

    public var body: some View {
        List {
            ForEach(model.sections) { section in
                Section {
                    ForEach(section.values) { row in
                        content(for: row)
                    }
                }
            }
        }
        .contentMargins(.top, .scene.top, for: .scrollContent)
        .listStyle(.insetGrouped)
        .listSectionSpacing(.compact)
        .navigationTitle(model.title)
        .taskOnce { Task { await model.load() }}
    }

    @ViewBuilder
    private func content(for row: AboutRowViewModel) -> some View {
        switch row.kind {
        case let .link(url):
            SafariNavigationLink(url: url) {
                ListItemView(model: row.model)
            }
        case .community:
            SocialLinksView(model: model.linksViewModel)
        case .version:
            ListItemView(model: row.model)
                .contextMenu(model.contextMenuItems)
            if let item = model.updateListItem {
                NavigationCustomLink(with: ListItemView(model: item), action: model.onUpdate)
            }
        }
    }
}
