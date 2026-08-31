// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Contacts
import GemstoneServices
import PrimitivesComponents
import Style
import SwiftUI

struct AddContactNavigationView: View {
    @Environment(\.contactService) private var contactService
    @Environment(\.nameService) private var nameService
    @Environment(\.addressService) private var addressService
    @Environment(\.chainService) private var chainService

    let action: AddContactType

    var body: some View {
        NavigationStack {
            Group {
                switch action {
                case let .new(recipient):
                    ManageContactScene(model: ManageContactViewModel(service: contactService, nameService: nameService, addressService: addressService, chainService: chainService, mode: .add(recipient)))
                case let .existing(recipient):
                    ContactsNavigationView(model: ContactsViewModel(service: contactService, nameService: nameService, addressService: addressService, chainService: chainService, mode: .addAddress(recipient)))
                }
            }
            .toolbarDismissItem(type: .close, placement: .cancellationAction)
        }
    }
}
