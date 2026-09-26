// Copyright (c). Gem Wallet. All rights reserved.

import Components
import struct Gemstone.GemConnectionDetails
import Localization
import Primitives
import PrimitivesComponents
import SwiftUI

struct ConnectionScene: View {
    @Environment(\.dismiss) private var dismiss
    let details: GemConnectionDetails
    let onDisconnect: () -> Void

    var body: some View {
        List {
            Section {
                ConnectionView(connection: details.connection)
            }
            Section {
                ForEach(details.rows, id: \.self) { row in
                    GemListRowView(row: row)
                }
            }
            Section {
                Button(Localized.WalletConnect.disconnect, role: .destructive) {
                    onDisconnect()
                    dismiss()
                }
            }
        }
        .listSectionSpacing(.compact)
        .navigationTitle(Localized.WalletConnect.Connection.title)
    }
}
