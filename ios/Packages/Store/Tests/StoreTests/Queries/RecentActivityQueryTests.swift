// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives
import PrimitivesTestKit
import Store
import StoreTestKit
import Testing

struct RecentActivityQueryTests {
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
    func recentAssetsAreNewestFirstAndOnePerAsset() throws {
        let db = DB.mock(wallets: [wallet], assets: walletAssets, balances: walletBalances)
        let store = RecentActivityStore(db: db)
        let btc = AssetId(chain: .bitcoin)
        let bnb = AssetId(chain: .smartChain)
        let now = Date()

        try store.add(assetId: btc, toAssetId: .none, walletId: WalletId.mock(), type: .search, createdAt: now.addingTimeInterval(-2))
        try store.add(assetId: bnb, toAssetId: .none, walletId: WalletId.mock(), type: .search, createdAt: now.addingTimeInterval(-1))
        try store.add(assetId: btc, toAssetId: .none, walletId: WalletId.mock(), type: .transfer, createdAt: now)

        try db.dbQueue.read { db in
            let result = try RecentActivityQuery(walletId: WalletId.mock(), limit: 10).fetch(db)

            #expect(result.count == 2)
            #expect(result.first?.asset.id == btc)
            #expect(result.last?.asset.id == bnb)
        }
    }

    @Test
    func filtersNarrowTheRecentAssets() throws {
        let db = DB.mock(wallets: [wallet], assets: walletAssets, balances: walletBalances)
        let store = RecentActivityStore(db: db)
        let assetStore = AssetStore(db: db)
        let btc = AssetId(chain: .bitcoin)
        let bnb = AssetId(chain: .smartChain)
        let eth = AssetId(chain: .ethereum)
        let walletId = WalletId.mock()

        try store.add(assetId: btc, toAssetId: .none, walletId: walletId, type: .search, createdAt: Date())
        try store.add(assetId: bnb, toAssetId: .none, walletId: walletId, type: .search, createdAt: Date())
        try store.add(assetId: eth, toAssetId: .none, walletId: walletId, type: .search, createdAt: Date())
        try assetStore.setAssetIsBuyable(for: [btc.identifier], value: false)

        try db.dbQueue.read { db in
            let noFilter = try RecentActivityQuery(walletId: walletId, limit: 10).fetch(db)
            let hasBalance = try RecentActivityQuery(walletId: walletId, limit: 10, filters: [.hasBalance]).fetch(db)
            let buyable = try RecentActivityQuery(walletId: walletId, limit: 10, filters: [.buyable]).fetch(db)
            let chains = try RecentActivityQuery(walletId: walletId, limit: 10, filters: [.chains([Chain.ethereum.rawValue])]).fetch(db)

            #expect(noFilter.count == 3)
            #expect(hasBalance.count == 2)
            #expect(hasBalance.map(\.asset.id).contains(btc) == false)
            #expect(buyable.count == 2)
            #expect(buyable.map(\.asset.id).contains(btc) == false)
            #expect(chains.count == 1)
            #expect(chains.first?.asset.id == eth)
        }
    }

    @Test
    func swapPayKeepsOnlyAssetsWithAnAvailableBalance() throws {
        let db = DB.mock(wallets: [wallet], assets: walletAssets, balances: walletBalances)
        let store = RecentActivityStore(db: db)
        let balanceStore = BalanceStore(db: db)
        let btc = AssetId(chain: .bitcoin)
        let bnb = AssetId(chain: .smartChain)
        let eth = AssetId(chain: .ethereum)
        let walletId = WalletId.mock()

        try store.add(assetId: btc, toAssetId: .none, walletId: walletId, type: .swap, createdAt: Date())
        try store.add(assetId: bnb, toAssetId: .none, walletId: walletId, type: .swap, createdAt: Date())
        try store.add(assetId: eth, toAssetId: .none, walletId: walletId, type: .swap, createdAt: Date())
        try balanceStore.setConfiguration(walletId: walletId, assetIds: [eth], configuration: .disabled)

        try db.dbQueue.read { db in
            let swapPay = try RecentActivityQuery(
                walletId: walletId,
                limit: 10,
                filters: [.enabled, .swappable, .hasAvailableBalance],
            ).fetch(db)
            let disabledBalance = try RecentActivityQuery(
                walletId: walletId,
                limit: 10,
                filters: [.disabledBalance],
            ).fetch(db)

            #expect(Set(swapPay.map(\.asset.id)) == Set([bnb, eth]))
            #expect(disabledBalance.map(\.asset.id) == [eth])
        }
    }
}
