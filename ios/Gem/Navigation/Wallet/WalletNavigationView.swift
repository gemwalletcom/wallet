// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Localization
import PrimitivesComponents
import Store
import SwiftUI
import Wallet

struct WalletNavigationView: View {
    @Environment(\.navigationRouter) private var navigationRouter
    @Environment(\.navigationState) private var navigationState
    @Environment(\.viewModelFactory) private var viewModelFactory

    @State private var model: WalletSceneViewModel

    init(model: WalletSceneViewModel) {
        _model = State(initialValue: model)
    }

    var body: some View {
        ZStack {
            WalletScene(model: model)
                .opacity(model.isPresentingSearch ? 0 : 1)

            if model.isPresentingSearch {
                WalletSearchScene(
                    model: viewModelFactory.walletSearchScene(
                        wallet: model.wallet,
                        onDismissSearch: model.onToggleSearch,
                        onSelectAssetAction: navigationRouter.openAsset,
                        onAddToken: model.onSelectAddCustomToken,
                    ),
                )
                .transition(.opacity)
            }
        }
        .onChange(of: navigationState.walletTabReselected, model.onWalletTabReselected)
        .bindQuery(
            model.walletQuery,
            model.assetsQuery,
            model.bannersQuery,
            model.fiatValuesQuery,
            model.perpetualBalanceQuery,
            model.collectionsQuery,
        )
        .navigationTitle(Localized.Wallet.title)
        .navigationBarTitleDisplayMode(.inline)
        .toolbar {
            ToolbarItem(placement: .principal) {
                if !model.isPresentingSearch {
                    WalletBarView(
                        row: model.walletRow,
                        action: model.onSelectWalletBar,
                    )
                    .liquidGlass()
                }
            }
            if !model.isPresentingSearch {
                ToolbarItem(placement: .navigationBarLeading) {
                    Button(action: model.onSelectScanner) {
                        model.scannerImage
                    }
                    .accessibilityIdentifier("scan")
                }
                ToolbarItem(placement: .navigationBarTrailing) {
                    Button(action: model.onToggleSearch) {
                        model.searchImage
                    }
                    .accessibilityIdentifier("search")
                }
            }
        }
        .scanReceiveSheet(isPresented: $model.isPresentingScanner, action: onScan)
        .safariSheet(url: $model.isPresentingUrl)
        .toast(message: $model.isPresentingToastMessage)
    }
}

// MARK: - Actions

extension WalletNavigationView {
    private func onScan(_ code: String) {
        Task { await navigationRouter.open(code: code) }
    }
}
