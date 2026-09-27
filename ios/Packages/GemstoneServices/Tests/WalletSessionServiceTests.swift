// Copyright (c). Gem Wallet. All rights reserved.

import class Gemstone.GemWalletSessionService
import GemstonePrimitives
@testable import GemstoneServices
import GemstoneServicesTestKit
import Primitives
import PrimitivesTestKit
import Store
import StoreTestKit
import Testing

struct WalletSessionServiceTests {
    @Test
    func currentWalletResolvesSelectedWalletAmongMany() async throws {
        let first = Wallet.mock(
            id: .mock(address: "0x1"),
            name: "First",
            accounts: [.mock(chain: .bitcoin, address: "bc1")],
        )
        let second = Wallet.mock(
            id: .mock(address: "0x2"),
            name: "Second",
            accounts: [.mock(chain: .ethereum, address: "0x2")],
        )
        let service = GemWalletSessionService.mock(store: WalletStore.mock(db: .mock(wallets: [first, second])))

        try service.setCurrentWalletId(walletId: second.id.id)

        let current = try await service.getCurrentWallet()?.toPrimitives()
        #expect(current?.id == second.id)
        #expect(current?.name == "Second")
        #expect(current?.accounts == second.accounts)
    }

    @Test
    func aWalletThatIsGoneReadsAsAnErrorInsteadOfAnEmptyScreen() async throws {
        let wallet = Wallet.mock(id: .mock(address: "0x1"), name: "First")
        let service = GemWalletSessionService.mock(store: WalletStore.mock(db: .mock(wallets: [wallet])))

        #expect(try await service.requireWallet(walletId: wallet.id.id).name == "First")
        await #expect(throws: (any Error).self) {
            try await service.requireWallet(walletId: WalletId.mock(address: "0xmissing").id)
        }
        await #expect(throws: (any Error).self) {
            try await service.requireCurrentWallet()
        }

        try service.setCurrentWalletId(walletId: wallet.id.id)
        #expect(try await service.requireCurrentWallet().id == wallet.id.id)
    }
}
