// Copyright (c). Gem Wallet. All rights reserved.

import class Gemstone.GemWalletService
import class Gemstone.GemWalletSessionService
import GemstoneServices
import GemstoneServicesTestKit
@testable import ManageWallets
import ManageWalletsTestKit
import Primitives
import PrimitivesComponents
import PrimitivesTestKit
@testable import Store
import StoreTestKit
import Testing

@MainActor
struct WalletsSceneViewModelTests {
    @Test
    func onDeleteConfirmed() async throws {
        let db = DB.mock(wallets: ["0x1", "0x2", "0x3"].map { .mock(id: .multicoin(address: $0)) })
        let walletStore = WalletStore.mock(db: db)

        let sessionStore = GemstoneWalletSessionStore.mock()
        let session = GemWalletSessionService.mock(store: walletStore, sessionStore: sessionStore)
        let service = GemWalletService.mock(db: db, sessionStore: sessionStore)
        try session.setCurrent(walletId: .multicoin(address: "0x1"))

        let model = WalletsSceneViewModel.mock(walletService: service)
        model.walletsQuery.value = try await session.getWallets().map(WalletEntry.init(wallet:))

        #expect(model.currentWalletId == .multicoin(address: "0x1"))

        await model.onDeleteConfirmed(wallet: .mock(id: .multicoin(address: "0x1")))

        #expect(model.currentWalletId == .multicoin(address: "0x2"))

        await model.onDeleteConfirmed(wallet: .mock(id: .multicoin(address: "0x2")))

        #expect(model.currentWalletId == .multicoin(address: "0x3"))

        await model.onDeleteConfirmed(wallet: .mock(id: .multicoin(address: "0x3")))

        #expect(model.currentWalletId == .none)
    }

    @Test
    func deletingAuthenticatesWhenAuthenticationIsEnabled() async throws {
        let wallet = Primitives.Wallet.mock(id: .multicoin(address: "0x1"))
        let db = DB.mock(wallets: [wallet])
        let biometry = BiometryAuthenticationMock()
        let model = WalletsSceneViewModel.mock(walletService: GemWalletService.mock(db: db), biometry: biometry)

        await model.onDeleteConfirmed(wallet: wallet)

        #expect(biometry.authenticateCallsCount == 1)
        #expect(try WalletStore.mock(db: db).getWallets().isEmpty)
    }

    @Test
    func cancellingAuthenticationKeepsTheWallet() async throws {
        let wallet = Primitives.Wallet.mock(id: .multicoin(address: "0x1"))
        let db = DB.mock(wallets: [wallet])
        let biometry = BiometryAuthenticationMock()
        biometry.authenticateError = BiometryAuthenticationError.cancelledByUser
        let model = WalletsSceneViewModel.mock(walletService: GemWalletService.mock(db: db), biometry: biometry)

        await model.onDeleteConfirmed(wallet: wallet)

        #expect(model.isPresentingAlertMessage == nil)
        #expect(try WalletStore.mock(db: db).getWallets().count == 1)
    }
}
