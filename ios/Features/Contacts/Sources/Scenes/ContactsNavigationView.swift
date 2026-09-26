// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Primitives
import Style
import SwiftUI

public struct ContactsNavigationView: View {
    @State private var model: ContactsSceneViewModel

    public init(model: ContactsSceneViewModel) {
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
                    contactEditor(for: model.addContactMode)
                        .toolbarDismissItem(type: .close, placement: .cancellationAction)
                }
            }
            .navigationDestination(for: Scenes.Contact.self) {
                contactEditor(for: .edit($0.contact))
            }
    }

    func contactEditor(for mode: ContactEditorSceneViewModel.Mode) -> some View {
        ContactEditorScene(model: model.contactEditorModel(mode: mode))
    }
}
