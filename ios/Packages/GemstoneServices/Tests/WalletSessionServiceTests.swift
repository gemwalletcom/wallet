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
        let service = try GemWalletSessionService.mock(store: WalletStore.mock(db: .mockWithWallets([first, second])))

        try service.setCurrent(walletId: second.id)

        let current = await service.currentWallet
        #expect(current?.id == second.id)
        #expect(current?.name == "Second")
        #expect(current?.accounts == second.accounts)
    }
}
