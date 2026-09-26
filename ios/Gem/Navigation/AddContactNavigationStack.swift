// Copyright (c). Gem Wallet. All rights reserved.

import Contacts
import GemstoneServices
import PrimitivesComponents
import SwiftUI

struct AddContactNavigationStack: View {
    @Environment(\.viewModelFactory) private var viewModelFactory

    let action: AddContactType

    var body: some View {
        NavigationStack {
            Group {
                switch action {
                case let .new(recipient, chain):
                    ContactEditorScene(model: viewModelFactory.contactEditorScene(mode: .add(recipient: recipient, chain: chain)))
                case let .existing(recipient, chain):
                    ContactsNavigationView(model: viewModelFactory.contactsScene(mode: .addAddress(recipient, chain: chain)))
                }
            }
            .toolbarDismissItem(type: .close, placement: .cancellationAction)
        }
    }
}
