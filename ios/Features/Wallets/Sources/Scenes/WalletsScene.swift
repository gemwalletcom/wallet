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
                        if let wallet = model.wallet(for: row) {
                            WalletListItemView(
                                wallet: wallet,
                                row: row,
                                onSelect: { model.onSelect(wallet: $0, dismiss: dismiss) },
                                onEdit: model.onEdit,
                                onPin: { wallet in Task { await model.onPin(wallet: wallet) } },
                                onDelete: model.onDelete,
                            )
                        }
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
            Localized.Common.deleteConfirmation(model.walletDelete?.name ?? ""),
            presenting: $model.walletDelete,
            sensoryFeedback: .warning,
            actions: { wallet in
                Button(
                    Localized.Common.delete,
                    role: .destructive,
                    action: { Task { await model.onDeleteConfirmed(wallet: wallet) } },
                )
            },
        )
        .navigationBarTitle(model.title)
        .bindQuery(model.walletsQuery)
        .onChange(of: model.hasWallets) {
            model.onChangeWallets(dismiss: dismiss)
        }
    }
}
