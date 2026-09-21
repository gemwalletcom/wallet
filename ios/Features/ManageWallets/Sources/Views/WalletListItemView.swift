// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import Localization
import Primitives
import PrimitivesComponents
import Style
import SwiftUI

struct WalletListItemView: View {
    let wallet: Wallet
    let listItem: ListItemModel
    let isPinned: Bool
    let currentWalletId: WalletId?

    let onSelect: (Wallet) -> Void
    let onEdit: (Wallet) -> Void
    let onPin: (Wallet) -> Void
    let onDelete: (Wallet) -> Void

    var body: some View {
        // https://www.jessesquires.com/blog/2023/07/18/navigation-link-accessory-view-swiftui
        // Hack to hide chevron
        ZStack {
            NavigationCustomLink(
                with: EmptyView(),
                action: { onSelect(wallet) },
            )
            .opacity(0)

            HStack {
                ListItemView(model: listItem)

                Spacer()

                if currentWalletId == wallet.id {
                    SelectionImageView()
                }

                Button(
                    action: { onEdit(wallet) },
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
                    action: { onEdit(wallet) },
                ),
                .pin(
                    isPinned: isPinned,
                    onPin: { onPin(wallet) },
                ),
                .delete { onDelete(wallet) },
            ],
        )
        .swipeActions {
            Button(
                action: { onEdit(wallet) },
                label: {
                    Label("", systemImage: SystemImage.settings)
                },
            )
            .tint(Colors.gray)
            Button(
                Localized.Common.delete,
                action: { onDelete(wallet) },
            )
            .tint(Colors.red)
        }
    }
}
