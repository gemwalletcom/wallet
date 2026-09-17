// Copyright (c). Gem Wallet. All rights reserved.

import Components
import SwiftUI

public struct AddressListItemView: View {
    @State private var isPresentingUrl: URL? = nil
    @State private var showAddress: Bool = false
    private let model: AddressListItemViewModel

    public init(model: AddressListItemViewModel) {
        self.model = model
    }

    public var body: some View {
        content
            .contextMenu(contextMenuItems)
            .safariSheet(url: $isPresentingUrl)
    }

    @ViewBuilder
    private var content: some View {
        if let onSelect = model.onSelect {
            NavigationCustomLink(with: listItem, action: onSelect)
        } else {
            listItem
                .onTap {
                    if model.canToggleAddress {
                        showAddress.toggle()
                    }
                }
        }
    }

    private var listItem: some View {
        ListItemImageView(
            title: model.title,
            subtitle: showAddress ? model.addressSubtitle : model.subtitle,
            assetImage: model.assetImage,
        )
    }

    private var contextMenuItems: [ContextMenuItemType] {
        var items: [ContextMenuItemType] = [
            .copy(value: model.account.address),
            .url(title: model.addressExplorerText, onOpen: { isPresentingUrl = model.addressExplorerUrl }),
        ]
        if let onAddContact = model.onAddContact {
            let recipient = model.addContactRecipient
            items.append(.custom(
                title: model.createContactTitle,
                systemImage: model.createContactImage,
                action: { onAddContact(.new(recipient, chain: model.account.chain)) },
            ))
            items.append(.custom(
                title: model.addToExistingContactTitle,
                systemImage: model.addToExistingContactImage,
                action: { onAddContact(.existing(recipient, chain: model.account.chain)) },
            ))
        }
        return items
    }
}
