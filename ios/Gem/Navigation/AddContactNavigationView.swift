// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Contacts
import ContactService
import PrimitivesComponents
import Style
import SwiftUI

struct AddContactNavigationView: View {
    @Environment(\.contactService) private var contactService
    @Environment(\.nameService) private var nameService

    let action: AddContactType

    var body: some View {
        NavigationStack {
            Group {
                switch action {
                case let .new(recipient):
                    ManageContactScene(model: ManageContactViewModel(service: contactService, nameService: nameService, mode: .add(recipient)))
                case let .existing(recipient):
                    ContactsNavigationView(model: ContactsViewModel(service: contactService, nameService: nameService, mode: .addAddress(recipient)))
                }
            }
            .toolbarDismissItem(type: .close, placement: .cancellationAction)
        }
    }
}
