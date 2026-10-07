// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Localization
import Style
import SwiftUI

struct PrivateKeyWarningScene: View {
    private let model: PrivateKeyExportViewModel

    init(model: PrivateKeyExportViewModel) {
        self.model = model
    }

    var body: some View {
        List {
            CalloutView(style: .header(title: model.warningMessage))
                .cleanListRow()

            ForEach(model.warningItems, id: \.self) { item in
                Section {
                    ListItemView(model: model.listItem(for: item))
                        .listRowInsets(.assetListRowInsets)
                }
            }
        }
        .safeAreaButton {
            StateButton(
                text: Localized.Common.continue,
                action: onContinue,
            )
        }
        .contentMargins([.top], .extraSmall, for: .scrollContent)
        .listSectionSpacing(.custom(.medium))
        .navigationTitle(model.title)
        .toolbarTitleDisplayMode(.inline)
        .toolbarInfoButton(url: model.docsUrl)
    }

    private func onContinue() {
        Task { await model.onContinue() }
    }
}
