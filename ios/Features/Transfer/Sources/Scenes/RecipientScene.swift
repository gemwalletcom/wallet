// Copyright (c). Gem Wallet. All rights reserved.

import Components
import func Gemstone.assetText
import Primitives
import PrimitivesComponents
import Store
import Style
import SwiftUI

public struct RecipientScene: View {
    public enum Field: Int, Hashable, Identifiable {
        case address
        case memo
        public var id: String {
            String(rawValue)
        }
    }

    @FocusState private var focusedField: Field?

    private var model: RecipientSceneViewModel

    public init(model: RecipientSceneViewModel) {
        self.model = model
    }

    public var body: some View {
        @Bindable var model = model
        List {
            Section {} header: {
                Group {
                    switch model.type {
                    case let .asset(asset):
                        AssetPreviewView(model: assetText(asset: asset))
                    case let .nft(nftAsset):
                        NftPreviewView(
                            assetImage: model.nftAssetImage(for: nftAsset.toPrimitives()),
                            name: nftAsset.name,
                            size: .image.large,
                        )
                    }
                }
                .frame(maxWidth: .infinity)
                .padding(.bottom, .small)
            }
            .cleanListRow()

            Section {
                AddressInputView(
                    model: $model.addressInputModel,
                    onSelectScan: { model.onSelectScan(field: .address) },
                )
                .focused($focusedField, equals: .address)
                .accessibilityIdentifier("addressInputField")
            }

            if model.showMemo {
                Section {
                    FloatTextField(
                        model.memoField,
                        text: $model.memo,
                        allowClean: focusedField == .memo,
                        trailingView: {
                            ListButton(
                                image: Images.System.qrCodeViewfinder,
                                action: { model.onSelectScan(field: .memo) },
                            )
                        },
                    )
                    .focused($focusedField, equals: .memo)
                    .accessibilityIdentifier("memoInputField")
                    .keyboardType(.alphabet)
                    .textInputAutocapitalization(.never)
                    .autocorrectionDisabled()
                }
            }

            ForEach(model.recipientSections) { section in
                Section {
                    ForEach(section.values) { row in
                        NavigationCustomLink(
                            with: ListItemView(model: model.listItem(for: row)),
                            action: {
                                focusedField = nil
                                model.onSelectRecipient(row.recipient)
                            },
                        )
                        .accessibilityIdentifier("recipient_row")
                    }
                } header: {
                    HStack {
                        if let image = section.image {
                            image
                                .frame(size: .image.small)
                        }
                        if let title = section.title {
                            Text(title)
                        }
                    }
                }
            }
        }
        .safeAreaButton {
            StateButton(
                text: model.actionButtonTitle,
                type: .primary(model.actionButtonState),
                action: onSelectContinue,
            )
        }
        .contentMargins(.top, .scene.top, for: .scrollContent)
        .listSectionSpacing(.compact)
        .navigationTitle(model.tittle)
        .toolbar {
            ToolbarItem(placement: .topBarTrailing) {
                Button(model.actionButtonTitle, action: onSelectContinue)
                    .bold()
                    .disabled(!model.isNextEnabled)
            }
        }
        .bindQuery(model.contactsQuery)
        .bindQuery(model.walletsQuery)
        .onChange(of: model.walletsQuery.value, initial: true) { model.updateRecipientSections() }
        .onChange(of: model.contactsQuery.value) { model.updateRecipientSections() }
        .onChange(of: model.addressInputModel.text, model.onChangeAddressText)
    }
}

// MARK: - Actions

extension RecipientScene {
    private func onSelectContinue() {
        focusedField = nil
        model.onContinue()
    }
}
