// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import Localization
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
    @State private var model: WalletDetailSceneViewModel

    public init(model: WalletDetailSceneViewModel) {
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
                                avatarImage: model.avatarAssetImage,
                                size: .image.extraLarge,
                                action: model.onSelectImage,
                            )
                            .padding(.bottom, .extraLarge)
                        }
                        Spacer()
                    }
                }
                if let secretKind = model.details.secretKind, let showSecret = model.details.showSecret {
                    Section {
                        NavigationCustomLink(
                            with: ListItemView(model: ListItemModel(title: showSecret.text)),
                            action: onShowSecret,
                        )
                    } header: {
                        Text(secretKind.title)
                    }
                }
                if let addressRow = model.details.address {
                    Section {
                        AddressListItemView(row: addressRow)
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
        .bindQuery(model.detailsQuery)
        .onChange(of: model.nameInput) { Task { await model.onChangeWalletName() } }
        .navigationTitle(model.title)
        .alert(
            model.details.row.deletePrompt.text,
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
            ExportWalletNavigationStack(flow: $0)
        }
    }
}

// MARK: - Actions

extension WalletDetailScene {
    private func onShowSecret() {
        Task { await model.onShowSecret() }
    }
}
