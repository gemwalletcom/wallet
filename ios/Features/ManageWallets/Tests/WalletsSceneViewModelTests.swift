// Copyright (c). Gem Wallet. All rights reserved.

import class Gemstone.GemWalletService
import class Gemstone.GemWalletSessionService
import GemstoneServices
import GemstoneServicesTestKit
@testable import ManageWallets
import ManageWalletsTestKit
import Primitives
import PrimitivesTestKit
@testable import Store
import StoreTestKit
import Testing

@MainActor
struct WalletsSceneViewModelTests {
    @Test
    func onDeleteConfirmed() async throws {
        let db = try DB.mockWithWallets(["0x1", "0x2", "0x3"].map { .mock(id: .multicoin(address: $0)) })
        let walletStore = WalletStore.mock(db: db)

        let sessionStore = GemstoneWalletSessionStore.mock()
        let session = GemWalletSessionService.mock(store: walletStore, sessionStore: sessionStore)
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
