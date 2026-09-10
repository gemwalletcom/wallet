// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Localization
import Primitives
import SwiftUI

struct ConnectionScene: View {
    @Environment(\.dismiss) private var dismiss
    let model: ConnectionSceneViewModel
    let onDisconnect: () -> Void

    var body: some View {
        List {
            Section {
                ConnectionView(model: model.model)
            }
            Section {
                ListItemView(title: model.walletField, subtitle: model.walletText)
                ListItemView(title: model.dateField, subtitle: model.dateText)
            }
            Section {
                Button(model.disconnectTitle, role: .destructive) {
                    onDisconnect()
                    dismiss()
                }
            }
        }
        .listSectionSpacing(.compact)
        .navigationTitle(model.title)
    }
}
