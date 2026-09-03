// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.GemContactAvatar
import struct Gemstone.GemContactAddressInput
import protocol Gemstone.GemManageContactServiceProtocol
import protocol Gemstone.GemNameServiceProtocol
import Components
import GemstoneServices
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

    enum Avatar {
        case empty
        case image(imageUrl: String)
        case emoji(EmojiValue)

        init(imageUrl: String?) {
            self = imageUrl.map { .image(imageUrl: $0) } ?? .empty
        }
    }

    private let service: any GemManageContactServiceProtocol
    private let nameService: any GemNameServiceProtocol
    private let mode: Mode

    let contactId: String

    var nameInputModel: InputValidationViewModel
    var description: String = ""
    var avatar: Avatar = .empty
    var addresses: [ContactAddress] = []
    var isPresentingAddress: ManageContactAddressViewModel.Mode?
    var isPresentingAvatar: Bool = false

    let emojiList: [EmojiValue] = Emoji.WalletAvatar.allCases.map { EmojiValue(emoji: $0.rawValue, color: Colors.grayVeryLight) }

    public init(
        service: any GemManageContactServiceProtocol,
        nameService: any GemNameServiceProtocol,
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
        case let .add(recipient):
            contactId = UUID().uuidString
            addresses = recipient.flatMap {
                try? GemContactAddressInput(
                    contactId: contactId,
                    chain: $0.chain,
                    address: $0.recipient.address,
                    memo: $0.recipient.memo,
                    replacingId: nil,
                ).addAddress([])
            } ?? []
        case let .edit(contactData):
            contactId = contactData.contact.id
            nameInputModel.text = contactData.contact.name
            description = contactData.contact.description ?? ""
            avatar = Avatar(imageUrl: contactData.contact.imageUrl)
            addresses = contactData.addresses
        }
    }

    var title: String {
        Localized.Contacts.contact
    }

    var defaultChain: Chain {
        service.defaultContactChain
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

    var avatarImage: AssetImage {
        switch avatar {
        case .empty: initials.isEmpty ? .image(Images.System.personCircleFill) : AssetImage(type: .text(initials))
        case let .image(imageUrl): AssetImage(type: .text(initials), imageURL: ImageSource(imageUrl).url)
        case let .emoji(value): AssetImage(type: .emoji(value.emoji))
        }
    }

    var avatarStyle: AssetImageView.Style? {
        switch avatar {
        case .empty: initials.isEmpty ? AssetImageView.Style(foregroundColor: Colors.grayLight) : nil
        case .image, .emoji: nil
        }
    }

    var onClearAvatar: VoidAction {
        switch avatar {
        case .empty: nil
        case .image, .emoji: { [weak self] in self?.avatar = .empty }
        }
    }

    func onSelectAvatar(_ value: EmojiValue) {
        avatar = .emoji(value)
        isPresentingAvatar = false
    }

    private func avatarInput() throws -> GemContactAvatar {
        switch avatar {
        case .empty:
            return .empty
        case let .image(imageUrl):
            return .image(imageUrl: imageUrl)
        case let .emoji(value):
            guard let data = EmojiAvatarRenderer.image(emoji: value.emoji, size: .image.extraLarge, color: value.color.uiColor).pngData() else {
                throw AnyError("Render avatar image failed")
            }
            return .rendered(image: data)
        }
    }

    private var initials: String {
        String(nameInputModel.text.trim().prefix(2))
    }

    func listItemModel(for address: ContactAddress) -> ListItemModel {
        ListItemModel(
            title: address.chain.networkName,
            titleExtra: service.formatAddress(address: address.address, chain: address.chain.rawValue, style: .short),
            imageStyle: .asset(assetImage: AssetIdViewModel(assetId: address.chain.assetId).assetImage),
        )
    }

    func addressModel(mode: ManageContactAddressViewModel.Mode) -> ManageContactAddressViewModel {
        ManageContactAddressViewModel(
            service: service,
            nameService: nameService,
            mode: mode,
            onComplete: { [weak self] in self?.onAddressComplete($0) },
        )
    }

    func onAddressComplete(_ input: ManageContactAddressViewModel.Input) {
        addresses = (try? GemContactAddressInput(
            contactId: contactId,
            chain: input.chain,
            address: input.address,
            memo: input.memo,
            replacingId: input.replacingId,
        ).addAddress(addresses)) ?? addresses
        isPresentingAddress = nil
    }

    func deleteAddress(at offsets: IndexSet) {
        addresses.remove(atOffsets: offsets)
    }

    func onSave() {
        Task {
            do {
                let contact = try await service.saveContact(
                    id: contactId,
                    existing: mode.contact,
                    name: nameInputModel.text,
                    description: description,
                    avatar: try avatarInput(),
                    addresses: addresses,
                )
                avatar = Avatar(imageUrl: contact.imageUrl)
            } catch {
                debugLog("ManageContactViewModel save error: \(error)")
            }
        }
    }
}
