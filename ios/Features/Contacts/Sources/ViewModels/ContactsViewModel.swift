// Copyright (c). Gem Wallet. All rights reserved.

import protocol Gemstone.GemManageContactServiceProtocol
import Components
import GemstoneServices
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

    enum RowAction {
        case navigate
        case select
    }

    private let service: any GemManageContactServiceProtocol
    private let mode: Mode

    public let query: ObservableQuery<ContactsRequest>
    var contacts: [ContactData] {
        query.value
    }

    var isPresentingAddContact = false

    public init(
        service: any GemManageContactServiceProtocol,
        mode: Mode = .list,
    ) {
        self.service = service
        self.mode = mode
        query = ObservableQuery(ContactsRequest(), initialValue: [])
    }

    var title: String {
        Localized.Contacts.title
    }

    var rowAction: RowAction {
        switch mode {
        case .list: .navigate
        case .addAddress: .select
        }
    }

    func manageContactModel(mode: ManageContactViewModel.Mode) -> ManageContactViewModel {
        ManageContactViewModel(service: service, mode: mode)
    }

    var addContactMode: ManageContactViewModel.Mode {
        switch mode {
        case .list: .add()
        case let .addAddress(recipient): .add(recipient)
        }
    }

    func add(to contact: ContactData) {
        guard case let .addAddress(recipient) = mode else { return }
        Task {
            do {
                let addresses = try service.addAddress(
                    contact.addresses,
                    contactId: contact.contact.id,
                    chain: recipient.chain,
                    address: recipient.recipient.address,
                    memo: recipient.recipient.memo,
                    replacingId: nil,
                )
                try await service.updateContact(contact.contact, addresses: addresses)
            } catch {
                debugLog("ContactsViewModel add error: \(error)")
            }
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
            imageStyle: .asset(assetImage: contact.contact.avatarImage),
        )
    }

    func deleteContacts(at offsets: IndexSet) {
        let selected = offsets.map { contacts[$0].contact }
        Task {
            do {
                for contact in selected {
                    try await service.deleteContact(contact)
                }
            } catch {
                debugLog("ContactsViewModel deleteContacts error: \(error)")
            }
        }
    }
}
