// Copyright (c). Gem Wallet. All rights reserved.

@testable import ManageWallets
import PreferencesTestKit
import Preferences
import Primitives
import PrimitivesTestKit
@testable import Store
import StoreTestKit
import SwiftUI
import Testing
import GemstoneServices
import GemstoneServicesTestKit

@MainActor
struct WalletsSceneViewModelTests {
    @Test
    func onDeleteConfirmed() async throws {
        let walletStore = WalletStore.mock(db: .mock())
        for address in ["0x1", "0x2", "0x3"] {
            try walletStore.addWallet(.mock(id: .multicoin(address: address)))
        }

        let preferences = ObservablePreferences.mock()
        let walletSessionService = WalletSessionService.mock(store: walletStore, preferences: preferences)
        let service = WalletService.mock(walletStore: walletStore, preferences: preferences)
        try walletSessionService.setCurrent(walletId: .multicoin(address: "0x1"))

        let model = WalletsSceneViewModel.mock(walletService: service, walletSessionService: walletSessionService)
        model.walletsQuery.value = walletSessionService.wallets

        #expect(model.currentWalletId == .multicoin(address: "0x1"))

        await model.onDeleteConfirmed(wallet: .mock(id: .multicoin(address: "0x1")))

        #expect(model.currentWalletId == .multicoin(address: "0x2"))

        await model.onDeleteConfirmed(wallet: .mock(id: .multicoin(address: "0x2")))

        #expect(model.currentWalletId == .multicoin(address: "0x3"))

        await model.onDeleteConfirmed(wallet: .mock(id: .multicoin(address: "0x3")))

        #expect(model.currentWalletId == .none)
    }
}

// MARK: - Mock Extensions

extension WalletsSceneViewModel {
    static func mock(
        navigationPath: Binding<NavigationPath> = .constant(NavigationPath()),
        walletService: WalletService = .mock(),
        walletSessionService: any WalletSessionManageable = WalletSessionService.mock(),
        isPresentingCreateWalletSheet: Binding<Bool> = .constant(false),
        isPresentingImportWalletSheet: Binding<Bool> = .constant(false),
    ) -> WalletsSceneViewModel {
        WalletsSceneViewModel(
            navigationPath: navigationPath,
            walletService: walletService,
            walletSessionService: walletSessionService,
            isPresentingCreateWalletSheet: isPresentingCreateWalletSheet,
            isPresentingImportWalletSheet: isPresentingImportWalletSheet,
        )
    }
}
