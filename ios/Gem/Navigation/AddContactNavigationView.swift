// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Contacts
import GemstoneServices
import PrimitivesComponents
import Style
import SwiftUI

struct AddContactNavigationView: View {
    @Environment(\.viewModelFactory) private var viewModelFactory

    let action: AddContactType

    var body: some View {
        NavigationStack {
            Group {
                switch action {
                case let .new(recipient):
                    ManageContactScene(model: viewModelFactory.manageContactScene(mode: .add(recipient)))
                case let .existing(recipient):
                    ContactsNavigationView(model: viewModelFactory.contactsScene(mode: .addAddress(recipient)))
                }
            }
            .toolbarDismissItem(type: .close, placement: .cancellationAction)
        }
    }
}
