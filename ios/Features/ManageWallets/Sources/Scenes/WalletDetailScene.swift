// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import Localization
import Onboarding
import Primitives
import PrimitivesComponents
import Store
import Style
import SwiftUI

public struct WalletDetailScene: View {
    enum Field: Int, Hashable {
        case name
    }

    @Environment(\.dismiss) private var dismiss
    @FocusState private var focusedField: Field?
    @State private var model: WalletDetailViewModel

    public init(model: WalletDetailViewModel) {
        _model = State(initialValue: model)
    }

    public var body: some View {
        VStack {
            List {
                Section {
                    FloatTextField(Localized.Wallet.name, text: $model.nameInput, allowClean: focusedField == .name)
                        .focused($focusedField, equals: .name)
                } header: {
                    HStack {
                        Spacer()
                        VStack(spacing: .medium) {
                            AvatarView(
                                avatarImage: model.avatarAssetImage(for: model.wallet),
                                size: .image.extraLarge,
                                action: model.onSelectImage,
                            )
                            .padding(.bottom, .extraLarge)
                        }
                        Spacer()
                    }
                }
                switch model.secretKind {
                case .phrase:
                    secretSection(Localized.Common.secretPhrase, action: model.onShowSecret)
                    if !model.privateKeyChains.isEmpty {
                        secretSection(Localized.Common.privateKey, action: model.onShowPrivateKey)
                    }
                case .privateKey:
                    secretSection(Localized.Common.privateKey, action: model.onShowSecret)
                case nil:
                    EmptyView()
                }
                Section {
                    switch model.address {
                    case let .account(account):
                        AddressListItemView(
                            model: AddressListItemViewModel(
                                title: Localized.Common.address,
                                account: account,
                                mode: .auto(addressStyle: .short),
                                addressLink: model.addressLink(account: account),
                            ),
                        )
                    case .none:
                        EmptyView()
                    }
                }
                Section {
                    HStack {
                        Spacer()
                        Button(role: .destructive, action: model.onSelectDelete) {
                            Text(Localized.Common.delete)
                                .foregroundStyle(Colors.red)
                        }
                        Spacer()
                    }
                }
            }
        }
        .padding(.bottom, .scene.bottom)
        .background(Colors.grayBackground)
        .frame(maxWidth: .infinity)
        .bindQuery(model.walletQuery)
        .onChange(of: model.nameInput) { Task { await model.onChangeWalletName() } }
        .navigationTitle(model.title)
        .alert(
            Localized.Common.deleteConfirmation(model.name),
            presenting: $model.isPresentingDeleteConfirmation,
            sensoryFeedback: .warning,
            actions: { _ in
                Button(
                    Localized.Common.delete,
                    role: .destructive,
                    action: {
                        Task {
                            if await model.onDelete() {
                                dismiss()
                            }
                        }
                    },
                )
            },
        )
        .alertSheet($model.isPresentingAlertMessage)
        .sheet(item: $model.isPresentingExportWallet) {
            ExportWalletNavigationStack(flow: $0, onExportPrivateKey: model.exportPrivateKey(chain:))
        }
    }

    private func secretSection(_ title: String, action: @escaping () -> Void) -> some View {
        Section {
            NavigationCustomLink(
                with: ListItemView(title: Localized.Common.show(title)),
                action: action,
            )
        } header: {
            Text(title)
        }
    }
}
