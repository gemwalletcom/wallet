// Copyright (c). Gem Wallet. All rights reserved.

import Primitives
import PrimitivesTestKit
@testable import Store
import StoreTestKit
import Testing

struct WalletStoreTests {
    @Test
    func anAccountNeedsItsChainAsset() throws {
        let store = WalletStore(db: .mock(chains: [.ethereum]))
        let wallet = Wallet.mock(accounts: [.mock(chain: .ethereum), .mock(chain: .solana)])

        #expect(throws: Error.self) {
            try store.addWallet(wallet)
        }

        try store.addWallet(.mock(accounts: [.mock(chain: .ethereum)]))

        #expect(try store.getWallets().count == 1)
    }
}
