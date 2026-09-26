// Copyright (c). Gem Wallet. All rights reserved.

import Components
import struct Gemstone.GemAddressRow
import Localization
import Primitives
import Style
import SwiftUI

public struct AddressListItemView: View {
    @State private var isPresentingUrl: URL? = nil
    @State private var showAddress: Bool = false
    private let row: GemAddressRow
    private let onSelect: (@MainActor @Sendable () -> Void)?
    private let onAddContact: ((AddContactType) -> Void)?

    public init(
        row: GemAddressRow,
        onSelect: (@MainActor @Sendable () -> Void)? = nil,
        onAddContact: ((AddContactType) -> Void)? = nil,
    ) {
        self.row = row
        self.onSelect = onSelect
        self.onAddContact = onAddContact
    }

    public var body: some View {
        content
            .contextMenu(contextMenuItems)
            .safariSheet(url: $isPresentingUrl)
    }

    @ViewBuilder
    private var content: some View {
        if row.isSelectable, let onSelect {
            NavigationCustomLink(with: listItem, action: onSelect)
        } else {
            listItem
                .onTap {
                    if row.shortAddress != nil {
                        showAddress.toggle()
                    }
                }
        }
    }

    private var listItem: some View {
        ListItemImageView(
            title: row.title.text,
            subtitle: showAddress ? row.shortAddress ?? row.text.text : row.text.text,
            assetImage: row.avatar?.assetImage,
        )
    }

    private var contextMenuItems: [ContextMenuItemType] {
        var items = row.menu.contextMenuItems { isPresentingUrl = $0 }
        if let contact = row.contact, let onAddContact {
            let chain = Chain(core: row.chain)
            items.append(.custom(
                title: Localized.Contacts.createNewContact,
                systemImage: SystemImage.personBadgePlus,
                action: { onAddContact(.new(contact, chain: chain)) },
            ))
            items.append(.custom(
                title: Localized.Contacts.addToExistingContact,
                systemImage: SystemImage.personCircle,
                action: { onAddContact(.existing(contact, chain: chain)) },
            ))
        }
        return items
    }
}
