// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives
import PrimitivesTestKit
import Store
import StoreTestKit
import Testing

struct RecentActivityStoreTests {
    @Test
    func clearWithoutAWalletForgetsTheTypeInEveryWallet() throws {
        let wallet = Wallet.mock()
        let otherWallet = Wallet.mock(id: .mock(address: "0x2"))
        let store = RecentActivityStore.mock(db: .mock(chains: [.bitcoin, .ethereum], wallets: [wallet, otherWallet]))
        try store.add(assetId: .mock(chain: .bitcoin), toAssetId: nil, walletId: wallet.id, type: .perpetual)
        try store.add(assetId: .mock(chain: .ethereum), toAssetId: nil, walletId: wallet.id, type: .search)
        try store.add(assetId: .mock(chain: .bitcoin), toAssetId: nil, walletId: otherWallet.id, type: .perpetual)

        try store.clear(scope: .allWallets, types: [.perpetual])

        #expect(try store.getRecent(walletId: wallet.id, types: RecentActivityType.allCases, limit: 10).map(\.asset.id) == [.mock(chain: .ethereum)])
        #expect(try store.getRecent(walletId: otherWallet.id, types: RecentActivityType.allCases, limit: 10).isEmpty)
    }
}
