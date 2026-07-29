// Copyright (c). Gem Wallet. All rights reserved.

import Components
import ContactService
import Foundation
import GemstonePrimitives
import Localization
import Primitives
import PrimitivesComponents
import Style
import SwiftUI
import UIKit
import Validators

@Observable
@MainActor
public final class ManageContactViewModel {
    public enum Mode {
        case add(ChainRecipient? = nil)
        case edit(ContactData)

        var contact: Contact? {
            switch self {
            case .add: nil
            case let .edit(contactData): contactData.contact
            }
        }
    }

    private let service: ContactService
    private let mode: Mode

    let contactId: String
    let nameService: any NameServiceable

    var nameInputModel: InputValidationViewModel
    var description: String = ""
    var avatar: String?
    var addresses: [ContactAddress] = []
    var isPresentingAddress: ManageContactAddressViewModel.Mode?
    var isPresentingAvatar: Bool = false

    let emojiList: [EmojiValue] = Emoji.WalletAvatar.allCases.map { EmojiValue(emoji: $0.rawValue, color: Colors.grayVeryLight) }

    public init(
        service: ContactService,
        nameService: any NameServiceable,
        mode: Mode,
    ) {
        self.service = service
        self.nameService = nameService
        self.mode = mode

        nameInputModel = InputValidationViewModel(
            mode: .onDemand,
            validators: [.required(requireName: Localized.Wallet.name)],
        )

        switch mode {
        case let .add(input):
            contactId = UUID().uuidString
            addresses = input.map {
                [ContactAddress.new(contactId: contactId, chain: $0.chain, address: $0.recipient.address, memo: $0.recipient.memo)]
            } ?? []
        case let .edit(contactData):
            contactId = contactData.contact.id
            nameInputModel.text = contactData.contact.name
            description = contactData.contact.description ?? ""
            avatar = contactData.contact.avatar
            addresses = contactData.addresses
        }
    }

    var title: String {
        Localized.Contacts.contact
    }

    var isAddMode: Bool {
        switch mode {
        case .add: true
        case .edit: false
        }
    }

    var buttonTitle: String {
        Localized.Common.save
    }

    var nameTitle: String {
        Localized.Wallet.name
    }

    var descriptionTitle: String {
        Localized.Common.description
    }

    var contactSectionTitle: String {
        Localized.Contacts.contact
    }

    var addressesSectionTitle: String {
        Localized.Contacts.addresses
    }

    var buttonState: ButtonState {
        guard nameInputModel.isValid,
              nameInputModel.text.isNotEmpty
        else {
            return .disabled
        }

        return .normal
    }

    let avatarSize: CGFloat = .image.extraLarge

    var avatarImage: AssetImage {
        currentContact.avatarImage
    }

    var hasAvatar: Bool {
        avatar != nil
    }

    var onClearAvatar: VoidAction {
        hasAvatar ? { [weak self] in self?.avatar = nil } : nil
    }

    private var currentContact: Contact {
        Contact.new(
            id: contactId,
            name: nameInputModel.text.trim(),
            description: description.isEmpty ? nil : description,
            avatar: avatar,
            createdAt: mode.contact?.createdAt ?? .now,
        )
    }

    func onSelectAvatar(_ value: EmojiValue) {
        avatar = value.emoji
        isPresentingAvatar = false
    }

    func listItemModel(for address: ContactAddress) -> ListItemModel {
        ListItemModel(
            title: address.chain.networkName,
            titleExtra: AddressFormatter(style: .short, address: address.address, chain: address.chain).value(),
            imageStyle: .asset(assetImage: AssetIdViewModel(assetId: address.chain.assetId).assetImage),
        )
    }

    func onAddressComplete(_ address: ContactAddress) {
        if let index = addresses.firstIndex(where: { $0.id == address.id }) {
            addresses[index] = address
        } else {
            addresses.append(address)
        }
        isPresentingAddress = nil
    }

    func deleteAddress(at offsets: IndexSet) {
        addresses.remove(atOffsets: offsets)
    }

    func onSave() {
        do {
            switch mode {
            case .add: try service.addContact(currentContact, addresses: addresses)
            case .edit: try service.updateContact(currentContact, addresses: addresses)
            }
        } catch {
            debugLog("ManageContactViewModel save error: \(error)")
        }
    }
}
