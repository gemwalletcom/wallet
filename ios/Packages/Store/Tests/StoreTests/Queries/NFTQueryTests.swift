// Copyright (c). Gem Wallet. All rights reserved.

import Primitives
import PrimitivesTestKit
import Store
import StoreTestKit
import Testing

struct NFTQueryTests {
    @Test
    func collectionsFollowTheWalletAssociation() throws {
        let owner = Wallet.mock(id: .mock(address: "0x1"), accounts: [.mock()])
        let other = Wallet.mock(id: .mock(address: "0x2"), accounts: [.mock()])
        let db = DB.mock(wallets: [owner, other])
        let store = NftStore.mock(db: db)
        let data = NFTData.mock(assets: [.mock()])

        try store.save([data], for: owner.id)

        try db.dbQueue.read { db in
            let owned = try NFTQuery(walletId: owner.id, filter: .all).fetch(db)
            let collection = try NFTQuery(walletId: owner.id, filter: .collection(id: data.collection.id.identifier)).fetch(db)
            let otherWallet = try NFTQuery(walletId: other.id, filter: .all).fetch(db)

            #expect(owned == [data])
            #expect(collection == [data])
            #expect(otherWallet.isEmpty)
        }

        try store.save([], for: owner.id)

        let sentAway = try db.dbQueue.read { try NFTQuery(walletId: owner.id, filter: .all).fetch($0) }

        #expect(sentAway.isEmpty)
    }
}
