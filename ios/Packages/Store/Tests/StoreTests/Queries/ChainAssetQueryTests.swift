// Copyright (c). Gem Wallet. All rights reserved.

import Primitives
import PrimitivesTestKit
import Store
import StoreTestKit
import Testing

struct ChainAssetQueryTests {
    @Test
    func nativeAsset() throws {
        let db = DB.mock(wallets: [.mock(accounts: [.mock(chain: .ethereum)])])
        let assetId = AssetId(chain: .ethereum)

        try db.dbQueue.read { db in
            let result = try ChainAssetQuery(walletId: .mock(), assetId: assetId, feeAssetId: assetId).fetch(db)

            #expect(result.assetData.asset.id == assetId)
            #expect(result.feeAssetData.asset.id == assetId)
        }
    }

    @Test
    func tokenWithoutBalance() throws {
        let token = Asset.mock(id: .mock(chain: .ethereum, tokenId: "0xdAC17F958D2ee523a2206206994597C13D831ec7"), name: "Tether", symbol: "USDT", decimals: 6, type: .erc20)
        let db = DB.mock(wallets: [.mock(accounts: [.mock(chain: .ethereum)])])
        try AssetStore(db: db).add(assets: [.mock(asset: token)])

        try db.dbQueue.read { db in
            let result = try ChainAssetQuery(walletId: .mock(), assetId: token.id, feeAssetId: token.chain.assetId).fetch(db)

            #expect(result.assetData.asset.id == token.id)
            #expect(result.assetData.balance == .zero)
            #expect(result.assetData.metadata.isBalanceEnabled == false)
            #expect(result.feeAssetData.asset.id == token.chain.assetId)
        }
    }
}
