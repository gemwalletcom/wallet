// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import ManageWallets
import Onboarding
import Primitives
import Style
import SwiftUI
import WalletSessionService

struct WalletsNavigationStack: View {
    @Environment(\.walletService) private var walletService
    @Environment(\.walletSessionService) private var walletSessionService
    @Environment(\.avatarService) private var avatarService
    @Environment(\.nameService) private var nameService
    @Environment(\.explorerService) private var explorerService
    @Environment(\.dismiss) private var dismiss

    @State private var navigationPath = NavigationPath()

    @State private var isPresentingCreateWalletSheet = false
    @State private var isPresentingImportWalletSheet = false

    var body: some View {
        NavigationStack(path: $navigationPath) {
            WalletsScene(
                model: WalletsSceneViewModel(
                    navigationPath: $navigationPath,
                    walletService: walletService,
                    walletSessionService: walletSessionService,
                    isPresentingCreateWalletSheet: $isPresentingCreateWalletSheet,
                    isPresentingImportWalletSheet: $isPresentingImportWalletSheet,
                ),
            )
            .navigationDestination(for: Scenes.WalletDetail.self) {
                WalletDetailScene(
                    model: WalletDetailViewModel(
                        navigationPath: $navigationPath,
                        wallet: $0.wallet,
                        walletService: walletService,
                        explorerService: explorerService,
                    ),
                )
            }
            .navigationDestination(for: Scenes.WalletSelectImage.self) {
                WalletImageScene(model: WalletImageViewModel(
                    wallet: $0.wallet,
                    avatarService: avatarService,
                ))
            }
            .sheet(isPresented: $isPresentingCreateWalletSheet) {
                CreateWalletNavigationStack(
                    model: CreateWalletModel(
                        walletService: walletService,
                        walletSessionService: walletSessionService,
                        avatarService: avatarService,
                        onComplete: { dismiss() },
                    ),
                )
            }
            .sheet(isPresented: $isPresentingImportWalletSheet) {
                ImportWalletNavigationStack(
                    model: ImportWalletViewModel(
                        walletService: walletService,
                        walletSessionService: walletSessionService,
                        avatarService: avatarService,
                        nameService: nameService,
                        onComplete: { dismiss() },
                    ),
                )
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
