// Copyright (c). Gem Wallet. All rights reserved.

import Components
import enum Gemstone.GemAboutRow
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
            ForEach(Array(model.sections.enumerated()), id: \.offset) { _, section in
                Section {
                    ForEach(section.rows, id: \.self) { row in
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
    private func content(for row: GemAboutRow) -> some View {
        switch row {
        case .termsOfService:
            SafariNavigationLink(url: model.termsOfServiceURL) {
                ListItemView(model: model.listItem(for: row))
            }
        case .privacyPolicy:
            SafariNavigationLink(url: model.privacyPolicyURL) {
                ListItemView(model: model.listItem(for: row))
            }
        case .website:
            SafariNavigationLink(url: model.websiteURL) {
                ListItemView(model: model.listItem(for: row))
            }
        case .community:
            SocialLinksView(model: model.linksViewModel)
        case .version:
            ListItemView(model: model.listItem(for: row))
                .contextMenu(model.contextMenuItems)

            if let item = model.updateListItem {
                NavigationCustomLink(with: ListItemView(model: item), action: model.onUpdate)
            }
        }
    }
}
