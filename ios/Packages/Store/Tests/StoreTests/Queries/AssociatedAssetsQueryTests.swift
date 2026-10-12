// Copyright (c). Gem Wallet. All rights reserved.

import Primitives
import PrimitivesTestKit
import Store
import StoreTestKit
import Testing

struct AssociatedAssetsQueryTests {
    @Test
    func assetsOfType() throws {
        let perpetual = Asset.mock(id: AssetId(chain: .hyperCore, tokenId: "perpetual::BTC"), type: .perpetual)
        let bitcoin = Asset.mock(id: AssetId(chain: .bitcoin))
        let wrapped = Asset.mock(id: AssetId(chain: .ethereum, tokenId: "0x2260FAC5E5542a773Aa44fBCfeDf7C193bc2C599"), type: .erc20)
        let db = DB.mock(assets: [.mock(asset: perpetual), .mock(asset: bitcoin), .mock(asset: wrapped)])
        try AssetStore(db: db).updateAssociations(assetId: perpetual.id, associations: [
            AssetAssociation(assetId: bitcoin.id, type: .official),
            AssetAssociation(assetId: wrapped.id, type: .wrapped),
        ])

        try db.dbQueue.read { db in
            let result = try AssociatedAssetsQuery(assetId: perpetual.id, type: .official).fetch(db)
            let notFound = try AssociatedAssetsQuery(assetId: bitcoin.id, type: .official).fetch(db)

            #expect(result == [bitcoin])
            #expect(notFound.isEmpty)
        }
    }
}
