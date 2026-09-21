// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import enum Gemstone.GemAddressFormatStyle
import class Gemstone.GemAddressService
import struct Gemstone.GemRecipient
import GemstonePrimitives
import Localization
import Primitives
import Style

public struct AddressListItemViewModel {
    public enum Mode {
        case auto(addressStyle: GemAddressFormatStyle)
        case address(addressStyle: GemAddressFormatStyle)
        case nameOrAddress
        case text(String)
    }

    public let title: String
    public let account: SimpleAccount
    public let mode: Mode
    public let onAddContact: ((AddContactType) -> Void)?
    public let onSelect: (@MainActor @Sendable () -> Void)?
    private let addressLink: BlockExplorerLink

    public init(
        title: String,
        account: SimpleAccount,
        mode: Mode,
        addressLink: BlockExplorerLink,
        onAddContact: ((AddContactType) -> Void)? = nil,
        onSelect: (@MainActor @Sendable () -> Void)? = nil,
    ) {
        self.title = title
        self.account = account
        self.mode = mode
        self.addressLink = addressLink
        self.onAddContact = onAddContact
        self.onSelect = onSelect
    }

    public var subtitle: String {
        switch mode {
        case let .auto(style): auto(for: style)
        case let .address(style): address(for: style)
        case .nameOrAddress: account.name ?? account.address
        case let .text(text): text
        }
    }

    public var assetImage: AssetImage? {
        account.assetImage
    }

    public var addressExplorerText: String {
        Localized.Transaction.viewOn(addressLink.name)
    }

    public var addressExplorerUrl: URL {
        addressLink.url
    }

    public var createContactTitle: String {
        Localized.Contacts.createNewContact
    }

    public var createContactImage: String {
        SystemImage.personBadgePlus
    }

    public var addToExistingContactTitle: String {
        Localized.Contacts.addToExistingContact
    }

    public var addToExistingContactImage: String {
        SystemImage.personCircle
    }

    public var addContactRecipient: GemRecipient {
        GemRecipient(address: account.address, memo: account.memo)
    }

    public var canToggleAddress: Bool {
        guard let name = account.name, name.isNotEmpty else {
            return false
        }
        return name != account.address
    }

    public var addressSubtitle: String {
        address(for: .short)
    }

    // MARK: - Private methods

    private func auto(for style: GemAddressFormatStyle) -> String {
        GemAddressService.shared.nameText(name: account.name, address: address(for: .short), hasImage: account.assetImage != nil) ?? address(for: style)
    }

    private func address(for style: GemAddressFormatStyle) -> String {
        GemAddressService.shared.format(address: account.address, chain: account.chain, style: style)
    }
}
