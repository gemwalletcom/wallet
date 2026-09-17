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
                ConnectionView(model: ConnectionViewModel(connection: model.details.connection))
            }
            Section {
                ForEach(model.details.rows, id: \.self) { row in
                    ListItemView(model: model.listItem(for: row))
                }
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
