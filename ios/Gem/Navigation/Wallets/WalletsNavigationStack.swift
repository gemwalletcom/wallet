// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import ManageWallets
import Onboarding
import Primitives
import Style
import SwiftUI
import GemstoneServices

struct WalletsNavigationStack: View {
    @Environment(\.viewModelFactory) private var viewModelFactory
    @Environment(\.dismiss) private var dismiss

    @State private var navigationPath = NavigationPath()

    @State private var isPresentingCreateWalletSheet = false
    @State private var isPresentingImportWalletSheet = false

    var body: some View {
        NavigationStack(path: $navigationPath) {
            WalletsScene(
                model: viewModelFactory.walletsScene(
                    navigationPath: $navigationPath,
                    isPresentingCreateWalletSheet: $isPresentingCreateWalletSheet,
                    isPresentingImportWalletSheet: $isPresentingImportWalletSheet,
                ),
            )
            .navigationDestination(for: Scenes.WalletDetail.self) {
                WalletDetailScene(model: viewModelFactory.walletDetailScene(navigationPath: $navigationPath, wallet: $0.wallet))
            }
            .navigationDestination(for: Scenes.WalletSelectImage.self) {
                WalletImageScene(model: viewModelFactory.walletImageScene(wallet: $0.wallet))
            }
            .sheet(isPresented: $isPresentingCreateWalletSheet) {
                CreateWalletNavigationStack(model: viewModelFactory.createWalletScene(onComplete: { dismiss() }))
            }
            .sheet(isPresented: $isPresentingImportWalletSheet) {
                ImportWalletNavigationStack(model: viewModelFactory.importWalletScene(onComplete: { dismiss() }))
            }
            .toolbar {
                ToolbarItem(placement: .topBarLeading) {
                    Button("", systemImage: SystemImage.xmark) {
                        dismiss()
                    }
                }
            }
            .navigationBarTitleDisplayMode(.inline)
        }
    }
}
