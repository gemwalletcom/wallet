// Copyright (c). Gem Wallet. All rights reserved.

import Primitives
import PrimitivesTestKit
import Store
import StoreTestKit
import Testing

struct ChainAssetQueryTests {
    private let wallet = Wallet.mock(accounts: [.mock(chain: .bitcoin), .mock(chain: .smartChain), .mock(chain: .tron), .mock(chain: .ethereum)])
    private let walletAssets: [AssetBasic] = [
        .mock(asset: .mock(name: "Bitcoin", symbol: "BTC", decimals: 8), properties: .mock(isEnabled: true, isBuyable: true, isSellable: true, isSwapable: true, isStakeable: true, stakingApr: 13.5, hasImage: true)),
        .mock(
            asset: .mock(id: .mock(chain: .smartChain), name: "BNB", symbol: "BNB", decimals: 18),
            properties: .mock(isEnabled: true, isBuyable: true, isSellable: true, isSwapable: true, isStakeable: true, stakingApr: 13.5, hasImage: true),
        ),
        .mock(asset: .mock(id: .mock(chain: .tron), name: "TRON", symbol: "TRX", decimals: 6), properties: .mock(isEnabled: true, isBuyable: true, isSellable: true, isSwapable: true, isStakeable: true, stakingApr: 13.5, hasImage: true)),
        .mock(
            asset: .mock(id: .mock(chain: .ethereum), name: "Ethereum", symbol: "ETH", decimals: 18),
            properties: .mock(isEnabled: true, isBuyable: true, isSellable: true, isSwapable: true, isStakeable: true, stakingApr: 13.5, hasImage: true),
        ),
        .mock(
            asset: .mock(id: .mock(chain: .ethereum, tokenId: "0xdAC17F958D2ee523a2206206994597C13D831ec7"), name: "Tether", symbol: "USDT", decimals: 6, type: .erc20),
            properties: .mock(isEnabled: true, isBuyable: true, isSellable: true, isSwapable: true, isStakeable: true, stakingApr: 13.5, hasImage: true),
        ),
    ]
    private let walletBalances: [UpdateBalance] = [
        .mock(assetId: .mock(chain: .smartChain), available: 1),
        .mock(assetId: .mock(chain: .tron), available: 2),
        .mock(assetId: .mock(chain: .ethereum), available: 3),
        .mock(assetId: .mock(chain: .ethereum, tokenId: "0xdAC17F958D2ee523a2206206994597C13D831ec7"), available: 4),
    ]

    @Test
    func fetchNativeAsset() throws {
        let db = DB.mock(wallets: [wallet], assets: walletAssets, balances: walletBalances)
        let assetId = AssetId(chain: .ethereum)

        try db.dbQueue.read { db in
            let result = try ChainAssetQuery(walletId: .mock(), assetId: assetId).fetch(db)

            #expect(result.assetData.asset.id == assetId)
            #expect(result.feeAssetData.asset.id == assetId)
        }
    }

    @Test
    func fetchToken() throws {
        let db = DB.mock(wallets: [wallet], assets: walletAssets, balances: walletBalances)
        let token = Asset.mock(id: .mock(chain: .ethereum, tokenId: "0xdAC17F958D2ee523a2206206994597C13D831ec7"), name: "Tether", symbol: "USDT", decimals: 6, type: .erc20)

        try db.dbQueue.read { db in
            let result = try ChainAssetQuery(walletId: .mock(), assetId: token.id).fetch(db)

            #expect(result.assetData.asset.id == token.id)
            #expect(result.feeAssetData.asset.id == token.chain.assetId)
        }
    }

    @Test
    func fetchTokenWithoutBalance() throws {
        let db = DB.mock(wallets: [wallet], assets: walletAssets, balances: walletBalances)
        let token = Asset.mock(id: .mock(chain: .ethereum, tokenId: "0xdAC17F958D2ee523a2206206994597C13D831ec7"), name: "Tether", symbol: "USDT", decimals: 6, type: .erc20)
        let balanceStore = BalanceStore(db: db)

        try balanceStore.deleteBalance(assetId: token.id)

        try db.dbQueue.read { db in
            let result = try ChainAssetQuery(walletId: .mock(), assetId: token.id).fetch(db)

            #expect(result.assetData.asset.id == token.id)
            #expect(result.assetData.balance == .zero)
            #expect(result.assetData.metadata.isBalanceEnabled == false)
            #expect(result.feeAssetData.asset.id == token.chain.assetId)
        }
    }
}
