// Copyright (c). Gem Wallet. All rights reserved.

import Components
import enum Gemstone.GemListRow
import Localization
import PrimitivesComponents
import SwiftUI

struct SignMessagePayloadSection: View {
    let primaryRows: [GemListRow]
    let secondaryRows: [GemListRow]
    let messageText: String
    let onSelectAddress: (String) -> Void
    let onViewDetails: () -> Void

    var body: some View {
        if primaryRows.isNotEmpty || secondaryRows.isNotEmpty {
            Section {
                ForEach(Array(primaryRows.enumerated()), id: \.offset) {
                    GemListRowView(row: $0.element, onSelectAddress: onSelectAddress)
                }

                if secondaryRows.isNotEmpty {
                    NavigationCustomLink(with: ListItemView(model: ListItemModel(title: Localized.Common.details)), action: onViewDetails)
                }
            }
        } else {
            Section(Localized.SignMessage.message) {
                Text(messageText)
            }
        }
    }
}
