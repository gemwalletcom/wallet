// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import struct Gemstone.GemWalletRow
import Localization
import Primitives
import PrimitivesComponents
import Style
import SwiftUI

struct WalletListItemView: View {
    let row: GemWalletRow

    let onSelect: (GemWalletRow) -> Void
    let onEdit: (GemWalletRow) -> Void
    let onPin: (GemWalletRow) -> Void
    let onDelete: (GemWalletRow) -> Void

    var body: some View {
        ZStack {
            NavigationCustomLink(
                with: EmptyView(),
                action: { onSelect(row) },
            )
            .opacity(0)

            HStack {
                ListItemView(model: row.listItem)

                Spacer()

                if row.isCurrent {
                    SelectionImageView()
                }

                Button(
                    action: { onEdit(row) },
                    label: {
                        Images.System.settings
                            .padding(.vertical, .small)
                            .padding(.leading, .small)
                    },
                )
                .buttonStyle(.borderless)
            }
        }
        .contextMenu(
            [
                .custom(
                    title: Localized.Settings.title,
                    systemImage: SystemImage.settings,
                    action: { onEdit(row) },
                ),
                .pin(
                    isPinned: row.isPinned,
                    onPin: { onPin(row) },
                ),
                .delete { onDelete(row) },
            ],
        )
        .swipeActions {
            Button(
                action: { onEdit(row) },
                label: {
                    Label("", systemImage: SystemImage.settings)
                },
            )
            .tint(Colors.gray)
            Button(
                Localized.Common.delete,
                action: { onDelete(row) },
            )
            .tint(Colors.red)
        }
    }
}
