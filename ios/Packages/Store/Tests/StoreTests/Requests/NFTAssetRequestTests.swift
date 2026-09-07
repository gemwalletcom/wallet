// Copyright (c). Gem Wallet. All rights reserved.

import Primitives
import PrimitivesTestKit
import Store
import StoreTestKit
import Testing

struct NFTAssetRequestTests {
    @Test
    func ownershipFollowsTheWalletAssociation() throws {
        let db = DB.mockWithChains([.bitcoin, .ethereum])
        let store = NftStore(db: db)
        let wallet = Wallet.mock()
        let walletId = wallet.id
        let assetData = NFTAssetData.mock(asset: .mock(chain: .ethereum))
        try WalletStore(db: db).addWallet(wallet)
        try store.add(asset: assetData.asset, collection: assetData.collection)

        try db.dbQueue.read { db in
            let details = try NFTAssetRequest(walletId: walletId, assetId: assetData.asset.id).fetch(db)
            #expect(details.assetData.asset.id == assetData.asset.id)
            #expect(details.assetData.collection.id == assetData.collection.id)
            #expect(details.isOwned == false)
        }

        try store.save([NFTData(collection: assetData.collection, assets: [assetData.asset])], for: walletId)

        try db.dbQueue.read { db in
            let held = try NFTAssetRequest(walletId: walletId, assetId: assetData.asset.id).fetch(db)
            let otherWallet = try NFTAssetRequest(walletId: .mock(address: "0x1"), assetId: assetData.asset.id).fetch(db)
            #expect(held.isOwned)
            #expect(otherWallet.isOwned == false)
        }
    }
}
