// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives
import PrimitivesComponents

public struct TransactionParticipantItemModel {
    public let title: String
    public let account: SimpleAccount
    public let addressLink: BlockExplorerLink
    public let onAddContact: ((AddContactType) -> Void)?
    public let onSelect: (@MainActor @Sendable () -> Void)?

    public init(
        title: String,
        account: SimpleAccount,
        addressLink: BlockExplorerLink,
        onAddContact: ((AddContactType) -> Void)? = nil,
        onSelect: (@MainActor @Sendable () -> Void)? = nil,
    ) {
        self.title = title
        self.account = account
        self.addressLink = addressLink
        self.onAddContact = onAddContact
        self.onSelect = onSelect
    }

    public var addressViewModel: AddressListItemViewModel {
        AddressListItemViewModel(
            title: title,
            account: account,
            mode: .nameOrAddress,
            addressLink: addressLink,
            onAddContact: onAddContact,
            onSelect: onSelect,
        )
    }
}
