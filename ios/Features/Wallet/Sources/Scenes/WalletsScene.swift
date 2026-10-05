// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Localization
import Primitives
import PrimitivesComponents
import Store
import Style
import SwiftUI

public struct WalletsScene: View {
    @Environment(\.dismiss) private var dismiss

    @State private var model: WalletsSceneViewModel

    public init(model: WalletsSceneViewModel) {
        _model = State(initialValue: model)
    }

    public var body: some View {
        List {
            Section {
                Button(
                    action: model.onSelectCreateWallet,
                    label: {
                        HStack {
                            Images.Wallets.create
                            Text(Localized.Wallet.createNewWallet)
                        }
                    },
                )
                Button(
                    action: model.onSelectImportWallet,
                    label: {
                        HStack {
                            Images.Wallets.import
                            Text(Localized.Wallet.importExistingWallet)
                        }
                    },
                )
            }

            ForEach(model.sections, id: \.kind) { section in
                Section {
                    ForEach(section.rows) { row in
                        WalletListItemView(
                            row: row,
                            onSelect: { model.onSelect(row: $0, dismiss: dismiss) },
                            onEdit: { row in Task { await model.onEdit(row: row) } },
                            onPin: { row in Task { await model.onPin(row: row) } },
                            onDelete: model.onDelete,
                        )
                    }
                } header: {
                    if let title = section.kind.title {
                        HStack {
                            section.kind.image
                            Text(title)
                        }
                    }
                }
            }
        }
        .contentMargins(.top, .scene.top, for: .scrollContent)
        .alertSheet($model.isPresentingAlertMessage)
        .alert(
            model.walletDeletePrompt,
            presenting: $model.walletDelete,
            sensoryFeedback: .warning,
            actions: { row in
                Button(
                    Localized.Common.delete,
                    role: .destructive,
                    action: { Task { await model.onDeleteConfirmed(row: row) } },
                )
            },
        )
        .navigationBarTitle(model.title)
        .bindQuery(model.walletsQuery)
        .onChange(of: model.walletsQuery.value, initial: true) {
            model.updateSections()
        }
        .onChange(of: model.currentWalletId) {
            model.updateSections()
        }
        .onChange(of: model.hasWallets) {
            model.onChangeWallets(dismiss: dismiss)
        }
    }
}
