// Copyright (c). Gem Wallet. All rights reserved.

import Components
import ContactService
import Foundation
import Localization
import Primitives
import PrimitivesComponents
import Store
import Style

@Observable
@MainActor
public final class ContactsViewModel {
    public enum Mode: Sendable {
        case list
        case addAddress(ChainRecipient)
    }

    let service: ContactService
    let nameService: any NameServiceable
    let mode: Mode

    public let query: ObservableQuery<ContactsRequest>
    var contacts: [ContactData] {
        query.value
    }

    var isPresentingAddContact = false

    public init(
        service: ContactService,
        nameService: any NameServiceable,
        mode: Mode = .list,
    ) {
        self.service = service
        self.nameService = nameService
        self.mode = mode
        query = ObservableQuery(ContactsRequest(), initialValue: [])
    }

    var title: String {
        Localized.Contacts.title
    }

    func add(to contact: ContactData) {
        guard case let .addAddress(recipient) = mode else { return }
        let updated = contact.addAddress(from: recipient)
        do {
            try service.updateContact(updated.contact, addresses: updated.addresses)
        } catch {
            debugLog("ContactsViewModel add error: \(error)")
        }
    }

    var emptyContent: EmptyContentTypeViewModel {
        EmptyContentTypeViewModel(type: .contacts)
    }

    func listItemModel(for contact: ContactData) -> ListItemModel {
        ListItemModel(
            title: contact.contact.name,
            titleStyle: TextStyle(font: .body, color: .primary, fontWeight: .semibold),
            titleExtra: contact.contact.description,
            titleStyleExtra: .calloutSecondary,
            titleExtraLineLimit: 1,
            imageStyle: .asset(assetImage: AssetImage(type: String(contact.contact.name.prefix(2)))),
        )
    }

    func deleteContacts(at offsets: IndexSet) {
        do {
            for index in offsets {
                try service.deleteContact(id: contacts[index].contact.id)
            }
        } catch {
            debugLog("ContactsViewModel deleteContacts error: \(error)")
        }
    }
}
