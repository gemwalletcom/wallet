// Copyright (c). Gem Wallet. All rights reserved.

import Components
import ContactService
import Primitives
import Style
import SwiftUI

public struct ContactsNavigationView: View {
    @State private var model: ContactsViewModel

    public init(model: ContactsViewModel) {
        _model = State(initialValue: model)
    }

    public var body: some View {
        ContactsScene(model: model)
            .bindQuery(model.query)
            .navigationTitle(model.title)
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .primaryAction) {
                    Button("", systemImage: SystemImage.plus, action: {
                        model.isPresentingAddContact = true
                    })
                }
            }
            .sheet(isPresented: $model.isPresentingAddContact) {
                NavigationStack {
                    manageContact(for: .add())
                        .toolbarDismissItem(type: .close, placement: .cancellationAction)
                }
            }
            .navigationDestination(for: Scenes.Contact.self) {
                manageContact(for: .edit($0.contact))
            }
    }

    func manageContact(for mode: ManageContactViewModel.Mode) -> some View {
        ManageContactScene(
            model: ManageContactViewModel(
                service: model.service,
                nameService: model.nameService,
                mode: mode,
            ),
        )
    }
}
