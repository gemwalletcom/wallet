// Copyright (c). Gem Wallet. All rights reserved.

import Primitives
import PrimitivesTestKit
import Store
import StoreTestKit
import Testing

struct NFTRequestTests {
    @Test
    func collectionsFollowTheWalletAssociation() throws {
        let owner = Wallet.mock(id: .mock(address: "0x1"), accounts: [.mock()])
        let other = Wallet.mock(id: .mock(address: "0x2"), accounts: [.mock()])
        let db = try DB.mockWithWallets([owner, other])
        let store = NftStore.mock(db: db)
        let data = NFTData.mock()

        try store.save([data], for: owner.id)

        try db.dbQueue.read { db in
            let owned = try NFTRequest(walletId: owner.id, filter: .all).fetch(db)
            let collection = try NFTRequest(walletId: owner.id, filter: .collection(id: data.collection.id.identifier)).fetch(db)
            let otherWallet = try NFTRequest(walletId: other.id, filter: .all).fetch(db)

            #expect(owned == [data])
            #expect(collection == [data])
            #expect(otherWallet.isEmpty)
        }

        try store.save([], for: owner.id)

        let sentAway = try db.dbQueue.read { try NFTRequest(walletId: owner.id, filter: .all).fetch($0) }

        #expect(sentAway.isEmpty)
    }
}
