// Copyright (c). Gem Wallet. All rights reserved.

import class Gemstone.GemWalletSessionService
import GemstonePrimitives
import Primitives
import PrimitivesTestKit
import Store
import StoreTestKit
import Testing
@testable import GemstoneServices
import GemstoneServicesTestKit

struct WalletSessionServiceTests {
    @Test
    func currentWalletResolvesSelectedWalletAmongMany() async throws {
        let store = WalletStore.mock(db: .mockWithChains([.bitcoin, .ethereum]))
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
        try store.addWallet(first)
        try store.addWallet(second)
        let service = GemWalletSessionService.mock(store: store)

        try service.setCurrent(walletId: second.id)

        let current = await service.currentWallet
        #expect(current?.id == second.id)
        #expect(current?.name == "Second")
        #expect(current?.accounts == second.accounts)
    }
}
