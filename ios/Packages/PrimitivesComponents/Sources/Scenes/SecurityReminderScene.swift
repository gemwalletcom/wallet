// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import Localization
import Primitives
import Style
import SwiftUI

public struct SecurityReminderScene: View {
    @State private var model: SecurityReminderViewModel

    public init(model: SecurityReminderViewModel) {
        self.model = model
    }

    public var body: some View {
        List {
            CalloutView(style: .header(title: model.message))
                .cleanListRow()

            ForEach(model.items, id: \.self) { item in
                Section {
                    ListItemView(model: model.listItem(for: item))
                        .listRowInsets(.assetListRowInsets)
                }
            }
        }
        .safeAreaButton {
            StateButton(
                text: Localized.Common.continue,
                action: model.onNext,
            )
        }
        .contentMargins([.top], .extraSmall, for: .scrollContent)
        .listSectionSpacing(.custom(.medium))
        .navigationTitle(model.title)
        .toolbarTitleDisplayMode(.inline)
        .toolbarInfoButton(url: model.docsUrl)
    }
}

#Preview {
    SecurityReminderScene(
        model: SecurityReminderViewModel(
            title: Localized.Wallet.New.title,
            onNext: {},
        ),
    )
}
