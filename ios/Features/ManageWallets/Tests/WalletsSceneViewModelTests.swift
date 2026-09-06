// Copyright (c). Gem Wallet. All rights reserved.

@testable import ManageWallets
import Primitives
import PrimitivesTestKit
@testable import Store
import StoreTestKit
import SwiftUI
import Testing
import GemstoneServices
import GemstoneServicesTestKit
import class Gemstone.GemWalletService
import class Gemstone.GemWalletService
import class Gemstone.GemWalletSessionService

@MainActor
struct WalletsSceneViewModelTests {
    @Test
    func onDeleteConfirmed() async throws {
        let db = DB.mock()
        let walletStore = WalletStore.mock(db: db)
        for address in ["0x1", "0x2", "0x3"] {
            try walletStore.addWallet(.mock(id: .multicoin(address: address)))
        }

        let sessionStore = GemstoneWalletSessionStore.mock()
        let session = GemWalletSessionService(store: sessionStore, wallets: GemstoneWalletStore(store: walletStore))
        let service = GemWalletService.mock(db: db, sessionStore: sessionStore)
        try session.setCurrent(walletId: .multicoin(address: "0x1"))

        let model = WalletsSceneViewModel.mock(walletService: service)
        model.walletsQuery.value = await session.wallets

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
        walletService: GemWalletService = .mock(),
        isPresentingCreateWalletSheet: Binding<Bool> = .constant(false),
        isPresentingImportWalletSheet: Binding<Bool> = .constant(false),
    ) -> WalletsSceneViewModel {
        WalletsSceneViewModel(
            navigationPath: navigationPath,
            walletService: walletService,
            preferences: .mock(),
            isPresentingCreateWalletSheet: isPresentingCreateWalletSheet,
            isPresentingImportWalletSheet: isPresentingImportWalletSheet,
        )
    }
}
