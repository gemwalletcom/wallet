// Copyright (c). Gem Wallet. All rights reserved.

import Components
import enum Gemstone.GemTransactionHeaderAction
import Localization
import NFT
import Primitives
import PrimitivesComponents
import SwiftUI
import Transactions

struct ActivityNavigationStack: View {
    @Environment(\.navigationRouter) private var navigationRouter
    @Environment(\.navigationState) private var navigationState
    @Environment(\.navigationPresenter) private var presenter
    @Environment(\.viewModelFactory) private var viewModelFactory

    let wallet: Wallet

    @State private var isPresentingToastMessage: ToastMessage?

    var body: some View {
        NavigationStack(path: navigationState.activity.binding) {
            TransactionsNavigationView(
                model: viewModelFactory.transactionsScene(wallet: wallet, type: .all),
            )
            .id(wallet.id)
            .navigationDestination(keyedBy: Scenes.Transaction.self, in: wallet.id) {
                TransactionNavigationView(model: viewModelFactory.transactionScene(
                    transactionId: $0.id,
                    wallet: wallet,
                    onHeaderAction: onHeaderAction,
                    onAddContact: { presenter.sheet.wrappedValue = .addContact($0) },
                    onSelectAddress: { presenter.sheet.wrappedValue = .addressDetails($0) },
                ))
            }
            .navigationDestination(keyedBy: Scenes.Collectible.self, in: wallet.id) {
                CollectibleScene(
                    model: viewModelFactory.collectibleScene(
                        wallet: wallet,
                        collectible: $0,
                        isPresentingSelectedAssetInput: presenter.isPresentingAssetInput,
                        onSelectAddress: { presenter.sheet.wrappedValue = .addressDetails($0) },
                    ),
                )
            }
        }
        .toast(message: $isPresentingToastMessage)
    }
}

// MARK: - Actions

extension ActivityNavigationStack {
    private func onHeaderAction(_ action: GemTransactionHeaderAction) {
        Task {
            do {
                try await navigationRouter.openTransactionAction(action, collectiblePath: navigationState.activity)
            } catch {
                isPresentingToastMessage = .error(Localized.Errors.errorOccurred)
            }
        }
    }
}
