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
    let service: ContactService
    let nameService: any NameServiceable

    public let query: ObservableQuery<ContactsRequest>
    var contacts: [ContactData] {
        query.value
    }

    var isPresentingAddContact = false

    public init(
        service: ContactService,
        nameService: any NameServiceable,
    ) {
        self.service = service
        self.nameService = nameService
        query = ObservableQuery(ContactsRequest(), initialValue: [])
    }

    var title: String {
        Localized.Contacts.title
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
