// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Localization
import Primitives
import PrimitivesComponents
import Style
import SwiftUI

public struct ContactEditorScene: View {
    @Environment(\.dismiss) private var dismiss

    @State private var model: ContactEditorSceneViewModel

    @FocusState private var focusedField: Field?
    enum Field: Int, Hashable {
        case name
        case description
    }

    public init(model: ContactEditorSceneViewModel) {
        _model = State(initialValue: model)
    }

    public var body: some View {
        List {
            avatarSection
            contactSection
            addressesSection
        }
        .listStyle(.insetGrouped)
        .listSectionSpacing(.compact)
        .contentMargins(.top, .scene.top, for: .scrollContent)
        .navigationTitle(model.title)
        .navigationBarTitleDisplayMode(.inline)
        .alertSheet($model.isPresentingAlertMessage)
        .toolbar {
            ToolbarItem(placement: .primaryAction) {
                Button("", systemImage: SystemImage.checkmark, action: onSave)
                    .disabled(model.buttonState == .disabled)
            }
        }
        .onChange(of: model.nameInputModel.text) { _, name in
            model.onChangeName(name)
        }
        .onAppear {
            if model.isAddMode {
                focusedField = .name
            }
        }
        .navigationDestination(for: Scenes.ContactAddress.self) {
            contactAddressScene(mode: .edit($0.address))
        }
        .sheet(item: $model.isPresentingAddress) { mode in
            NavigationStack {
                contactAddressScene(mode: mode)
                    .toolbarDismissItem(type: .close, placement: .cancellationAction)
            }
        }
        .sheet(isPresented: $model.isPresentingAvatar) {
            NavigationStack {
                EmojiSelectorView(emojis: model.emojiList, onSelect: model.onSelectAvatar)
                    .navigationTitle(Localized.Common.emoji)
                    .navigationBarTitleDisplayMode(.inline)
                    .toolbarDismissItem(type: .close, placement: .cancellationAction)
                    .background(Colors.grayBackground)
            }
        }
    }

    private func contactAddressScene(mode: ContactAddressEditorSceneViewModel.Mode) -> some View {
        ContactAddressEditorScene(model: model.addressModel(mode: mode))
    }
}

// MARK: - UI Components

extension ContactEditorScene {
    private var avatarSection: some View {
        Section {
            AvatarView(
                avatarImage: model.avatarImage,
                size: .image.extraLarge,
                action: onSelectAvatar,
                removeAction: model.onClearAvatar,
                style: model.avatarStyle,
            )
            .frame(maxWidth: .infinity, alignment: .center)
            .listRowBackground(Color.clear)
        }
    }

    private var contactSection: some View {
        Section {
            InputValidationField(
                model: $model.nameInputModel,
                placeholder: model.nameTitle,
                allowClean: true,
            )
            .focused($focusedField, equals: .name)
            .textInputAutocapitalization(.words)

            FloatTextField(
                model.descriptionTitle,
                text: $model.description,
                allowClean: true,
            )
            .focused($focusedField, equals: .description)
        }
    }

    private var addressesSection: some View {
        Section {
            ForEach(model.addressRows, id: \.address.id) { row in
                NavigationLink(value: Scenes.ContactAddress(address: row.address.toPrimitives())) {
                    ListItemView(model: row.listItem)
                }
            }
            .onDelete(perform: model.deleteAddress)

            Button(action: onAddAddress) {
                HStack {
                    Images.System.plus
                    Text(Localized.Common.address)
                }
            }
        } header: {
            Text(model.addressesSectionTitle)
        }
    }
}

// MARK: - Actions

extension ContactEditorScene {
    private func onAddAddress() {
        focusedField = .none
        model.isPresentingAddress = .add
    }

    private func onSelectAvatar() {
        focusedField = .none
        model.isPresentingAvatar = true
    }

    private func onSave() {
        focusedField = .none
        model.onSave(dismiss: dismiss)
    }
}
