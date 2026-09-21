// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import func Gemstone.contactRow
import struct Gemstone.GemContactAddressInput
import protocol Gemstone.GemContactServiceProtocol
import struct Gemstone.GemRecipient
import GemstonePrimitives
import GemstoneServices
import Localization
import Primitives
import PrimitivesComponents
import Store
import Style
import SwiftUI

@Observable
@MainActor
public final class ContactsViewModel {
    public enum Mode: Sendable {
        case list
        case addAddress(GemRecipient, chain: Chain)
    }

    enum RowAction {
        case navigate
        case select
    }

    private let service: any GemContactServiceProtocol
    private let contactEditor: @MainActor (ContactEditorViewModel.Mode) -> ContactEditorViewModel
    private let mode: Mode

    var isPresentingAlertMessage: AlertMessage?

    public let query: ObservableQuery<ContactsRequest>
    var contacts: [ContactData] {
        query.value
    }

    var isPresentingAddContact = false

    public init(
        service: any GemContactServiceProtocol,
        contactEditor: @escaping @MainActor (ContactEditorViewModel.Mode) -> ContactEditorViewModel,
        mode: Mode = .list,
    ) {
        self.service = service
        self.contactEditor = contactEditor
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

    func contactEditorModel(mode: ContactEditorViewModel.Mode) -> ContactEditorViewModel {
        contactEditor(mode)
    }

    var addContactMode: ContactEditorViewModel.Mode {
        switch mode {
        case .list: .add()
        case let .addAddress(recipient, chain): .add(recipient: recipient, chain: chain)
        }
    }

    func onSelect(contact: ContactData, dismiss: DismissAction) {
        guard case let .addAddress(recipient, chain) = mode else { return }
        Task {
            do {
                let addresses = GemContactAddressInput(
                    contactId: contact.contact.id,
                    chain: chain,
                    address: recipient.address,
                    memo: recipient.memo,
                    replacingId: nil,
                ).addAddress(contact.addresses)
                try await service.updateContact(contact.contact, addresses: addresses)
                dismiss()
            } catch {
                isPresentingAlertMessage = AlertMessage(error: error)
            }
        }
    }

    var emptyContent: EmptyContentTypeViewModel {
        EmptyContentTypeViewModel(type: .contacts)
    }

    func listItemModel(for contact: ContactData) -> ListItemModel {
        let row = contactRow(contact: contact.contact.toGem())
        return ListItemModel(
            title: row.title,
            titleExtra: row.subtitle,
            titleStyleExtra: .calloutSecondary,
            titleExtraLineLimit: 1,
            imageStyle: .asset(assetImage: contact.contact.avatarImage(initials: row.initials)),
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
                isPresentingAlertMessage = AlertMessage(error: error)
            }
        }
    }
}
