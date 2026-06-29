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
        ListItemImageView(
            title: model.title,
            subtitle: showAddress ? model.addressSubtitle : model.subtitle,
            assetImage: model.assetImage,
            assetImageStyle: model.assetImageStyle,
            imageSize: model.assetImageSize,
        )
        .onTap {
            if model.canToggleAddress {
                showAddress.toggle()
            }
        }
        .contextMenu(contextMenuItems)
        .safariSheet(url: $isPresentingUrl)
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
                action: { onAddContact(.new(recipient)) },
            ))
            items.append(.custom(
                title: model.addToExistingContactTitle,
                systemImage: model.addToExistingContactImage,
                action: { onAddContact(.existing(recipient)) },
            ))
        }
        return items
    }
}
