// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import struct Gemstone.GemContactAddressInput
import struct Gemstone.GemContactAddressRow
import enum Gemstone.GemContactAvatar
import enum Gemstone.GemContactAvatarChoice
import protocol Gemstone.GemContactEditorServiceProtocol
import struct Gemstone.GemContactSession
import protocol Gemstone.GemNameServiceProtocol
import struct Gemstone.GemRecipient
import GemstonePrimitives
import GemstoneServices
import Localization
import Primitives
import PrimitivesComponents
import Style
import SwiftUI
import UIKit

@Observable
@MainActor
public final class ContactEditorSceneViewModel {
    public enum Mode {
        case add(recipient: GemRecipient? = nil, chain: Chain? = nil)
        case edit(ContactData)
    }

    private let service: any GemContactEditorServiceProtocol
    private let nameService: any GemNameServiceProtocol
    private let mode: Mode

    private(set) var session: GemContactSession
    var nameInputModel: InputValidationViewModel
    var isPresentingAddress: ContactAddressEditorSceneViewModel.Mode?
    var isPresentingAvatar: Bool = false
    var isPresentingAlertMessage: AlertMessage?

    let emojiList: [EmojiValue] = GemConstants.walletAvatarEmojis.map { EmojiValue(emoji: $0, color: Colors.grayVeryLight) }

    public init(
        service: any GemContactEditorServiceProtocol,
        nameService: any GemNameServiceProtocol,
        mode: Mode,
    ) {
        self.service = service
        self.nameService = nameService
        self.mode = mode

        nameInputModel = InputValidationViewModel()

        switch mode {
        case let .add(recipient, chain):
            let session = service.newSession(contact: nil, addresses: [])
            self.session = recipient.flatMap { recipient in
                chain.map {
                    session.onAddressSaved(
                        input: GemContactAddressInput(contactId: session.id, chain: $0.rawValue, address: recipient.address, memo: recipient.memo, replacingId: nil),
                    )
                }
            } ?? session
        case let .edit(contactData):
            session = service.newSession(
                contact: contactData.contact.toGem(),
                addresses: contactData.addresses.map { $0.toGem() },
            )
            nameInputModel.text = contactData.contact.name
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

    var addressesSectionTitle: String {
        Localized.Contacts.addresses
    }

    var description: String {
        get { session.description }
        set { session = session.onDescriptionChanged(description: newValue) }
    }

    var addressRows: [GemContactAddressRow] {
        session.addressRows()
    }

    var buttonState: ButtonState {
        guard session.canSave() else {
            return .disabled
        }
        return session.isSaving ? .loading(showProgress: true) : .normal
    }

    var avatarImage: AssetImage {
        session.avatarImage().assetImage
    }

    var avatarStyle: AssetImageView.Style? {
        session.avatarImage().style
    }

    var onClearAvatar: VoidAction {
        switch session.avatar {
        case .empty: nil
        case .image, .emoji: { [weak self] in self?.onChangeAvatar(.empty) }
        }
    }

    func onChangeName(_ name: String) {
        session = session.onNameChanged(name: name)
    }

    func onSelectAvatar(_ value: EmojiValue) {
        onChangeAvatar(.emoji(emoji: value.emoji))
        isPresentingAvatar = false
    }

    private func onChangeAvatar(_ avatar: GemContactAvatarChoice) {
        session = session.onAvatarChanged(avatar: avatar)
    }

    private func avatarInput() throws -> GemContactAvatar {
        switch session.avatar {
        case .empty:
            return .empty
        case let .image(imageUrl):
            return .image(imageUrl: imageUrl)
        case let .emoji(emoji):
            guard let data = EmojiAvatarRenderer.image(emoji: emoji, size: .image.extraLarge, color: Colors.grayVeryLight.uiColor).pngData() else {
                throw AnyError("Render avatar image failed")
            }
            return .rendered(image: data)
        }
    }

    func addressModel(mode: ContactAddressEditorSceneViewModel.Mode) -> ContactAddressEditorSceneViewModel {
        ContactAddressEditorSceneViewModel(
            service: service,
            nameService: nameService,
            contactId: session.id,
            mode: mode,
            onComplete: { [weak self] in self?.onAddressComplete($0) },
        )
    }

    func onAddressComplete(_ input: GemContactAddressInput) {
        session = session.onAddressSaved(input: input)
        isPresentingAddress = nil
    }

    func deleteAddress(at offsets: IndexSet) {
        for id in offsets.map({ session.addresses[$0].id }) {
            session = session.onAddressDeleted(addressId: id)
        }
    }

    func onSave(dismiss: DismissAction) {
        session = session.onSaving(isSaving: true)
        Task {
            defer { session = session.onSaving(isSaving: false) }
            do {
                _ = try await service.saveContact(input: session.input(avatar: avatarInput()))
                dismiss()
            } catch {
                isPresentingAlertMessage = AlertMessage(error: error)
            }
        }
    }
}
