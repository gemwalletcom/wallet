// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Primitives
import PrimitivesComponents
import Style
import SwiftUI

public struct ContactsScene: View {
    @Environment(\.dismiss) private var dismiss

    @State private var model: ContactsSceneViewModel

    public init(model: ContactsSceneViewModel) {
        _model = State(initialValue: model)
    }

    public var body: some View {
        List {
            ForEach(model.items, id: \.contact.id) { contact, listItem in
                let item = ListItemView(model: listItem)
                switch model.rowAction {
                case .navigate:
                    NavigationLink(value: Scenes.Contact(contact: contact)) { item }
                case .select:
                    Button {
                        model.onSelect(contact: contact, dismiss: dismiss)
                    } label: { item }
                        .buttonStyle(.plain)
                }
            }
            .onDelete(perform: model.deleteContacts)
        }
        .contentMargins(.top, .scene.top, for: .scrollContent)
        .listStyle(.insetGrouped)
        .listSectionSpacing(.compact)
        .scrollContentBackground(.hidden)
        .background { Colors.insetGroupedListStyle.ignoresSafeArea() }
        .overlay {
            if model.contacts.isEmpty {
                EmptyContentView(model: model.emptyContent)
            }
        }
        .alertSheet($model.isPresentingAlertMessage)
    }
}
