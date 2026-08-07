// Copyright (c). Gem Wallet. All rights reserved.

import Primitives
import PrimitivesTestKit
import Store
import StoreTestKit
import Testing
@testable import WalletSessionService
import WalletSessionServiceTestKit

struct WalletSessionServiceTests {
    @Test
    func setCurrentReturnsWalletId() throws {
        let wallet = Wallet.mock(index: 1)
        let service = try WalletSessionService.mock(wallet: wallet)

        #expect(service.setCurrent(index: 1) == wallet.id)
        #expect(service.currentWalletId == wallet.id)
    }

    @Test
    func setCurrentReturnsNil() throws {
        let service = try WalletSessionService.mock(wallet: .mock(index: 1))

        #expect(service.setCurrent(index: 999) == .none)
        #expect(service.currentWalletId == .none)
    }

    @Test
    func currentWalletResolvesSelectedWalletAmongMany() throws {
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
        let service = WalletSessionService.mock(store: store)

        service.setCurrent(walletId: second.id)

        #expect(service.currentWallet?.id == second.id)
        #expect(service.currentWallet?.name == "Second")
        #expect(service.currentWallet?.accounts == second.accounts)
    }
}
