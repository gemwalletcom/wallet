// Copyright (c). Gem Wallet. All rights reserved.

import Primitives
import PrimitivesTestKit
import Store
import StoreTestKit
import Testing

struct AssetsQueryTests {
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

    @Test func excludesNegativeRank() throws {
        let visible = AssetBasic.mock(asset: .mock(id: AssetId(chain: .ethereum)), score: .mock(rank: 0))
        let hidden = AssetBasic.mock(asset: .mock(id: AssetId(chain: .tempo)), score: .mock(rank: -1))
        let db = DB.mock(wallets: [.mock(accounts: [visible, hidden].map { .mock(chain: $0.asset.chain) })], assets: [visible, hidden])

        try db.dbQueue.read { db in
            let assets = try AssetsQuery.mock().fetch(db)
            let priceAlertAssets = try AssetsQuery.mock(scope: .allAssets).fetch(db)

            #expect(assets.map(\.asset.id) == [visible.asset.id])
            #expect(priceAlertAssets.map(\.asset.id) == [visible.asset.id])
        }
    }

    @Test func addAssets() throws {
        let db = DB.mock()
        let store = AssetStore(db: db)

        try db.dbQueue.read { db in
            let assets = try AssetsQuery.mock().fetch(db)

            #expect(assets.isEmpty)
        }

        try store.add(assets: [.mock()])

        try db.dbQueue.read { db in
            let assets = try AssetsQuery.mock(scope: .allAssets).fetch(db)

            #expect(assets.count == 1)
        }
    }

    @Test func assetDataIncludesAssociations() throws {
        let asset = AssetBasic.mock()
        let db = DB.mock(wallets: [.mock(accounts: [.mock(chain: asset.asset.chain)])], assets: [asset])
        let store = AssetStore(db: db)
        let associations = [AssetAssociation(assetId: AssetId(chain: .ethereum), type: .official)]

        try store.updateAssociations(assetId: asset.asset.id, associations: associations)
        try store.add(assets: [asset])

        let storedAsset = try db.dbQueue.read { db in
            try AssetQuery(walletId: .mock(), assetId: asset.asset.id).fetch(db)
        }
        #expect(storedAsset.associations == associations)
    }

    @Test func pinned() throws {
        let db = DB.mock(wallets: [wallet], assets: walletAssets, balances: walletBalances)
        let balanceStore = BalanceStore(db: db)

        let assetId = AssetId(chain: .bitcoin, tokenId: nil)
        try balanceStore.setConfiguration(walletId: .mock(), assetIds: [assetId], configuration: .pinned(true))

        try db.dbQueue.read { db in
            let assets = try AssetsQuery.mock().fetch(db)

            #expect(assets.first?.asset.id == assetId)
            #expect(assets.first?.metadata.isPinned == true)
            #expect(assets.first?.metadata.isBalanceEnabled == true)
        }
    }

    @Test func enabled() throws {
        let db = DB.mock(wallets: [wallet], assets: walletAssets, balances: walletBalances)
        let balanceStore = BalanceStore(db: db)

        let disabledId = AssetId(chain: .bitcoin)
        try balanceStore.setConfiguration(walletId: .mock(), assetIds: [disabledId], configuration: .disabled)

        try db.dbQueue.read { db in
            let enabledAssets = try AssetsQuery.mock(filters: [.enabledBalance]).fetch(db)
            let enabledIds = enabledAssets.map(\.asset.id)

            #expect(enabledAssets.count == 4)
            #expect(enabledIds.contains(disabledId) == false)
        }
    }

    @Test func assetProperties() throws {
        let db = DB.mock(wallets: [wallet], assets: walletAssets, balances: walletBalances)
        let assetStore = AssetStore(db: db)

        let assetId = AssetId(chain: .bitcoin)
        try assetStore.setAssetIsBuyable(for: [assetId.identifier], value: false)
        try assetStore.setAssetIsSwappable(for: [assetId.identifier], value: false)

        try db.dbQueue.read { db in
            let assets = try AssetsQuery.mock(filters: [.buyable, .swappable]).fetch(db)

            #expect(assets.count == 4)
            #expect(assets.map(\.asset.id).contains(assetId) == false)
        }
    }

    @Test func testHasBalance() throws {
        let db = DB.mock(wallets: [wallet], assets: walletAssets, balances: walletBalances)

        try db.dbQueue.read { db in
            let assets = try AssetsQuery.mock(filters: [.hasBalance]).fetch(db)

            #expect(assets.count == 4)
            #expect(assets.first?.asset.id == AssetId(chain: .smartChain))
        }
    }

    @Test func testChains() throws {
        let db = DB.mock(wallets: [wallet], assets: walletAssets, balances: walletBalances)

        try db.dbQueue.read { db in
            let ethereumAssets = try AssetsQuery.mock(filters: [.chains([Chain.ethereum.rawValue, Chain.solana.rawValue])]).fetch(db)
            let bitcoinAssets = try AssetsQuery.mock(filters: [.chains([Chain.bitcoin.rawValue, Chain.base.rawValue])]).fetch(db)

            #expect(ethereumAssets.count == 2)
            #expect(bitcoinAssets.count == 1)
        }
    }

    @Test func testChainsOrAssets() throws {
        let db = DB.mock(wallets: [wallet], assets: walletAssets, balances: walletBalances)

        try db.dbQueue.read { db in
            let assets = try AssetsQuery.mock(
                filters: [.chainsOrAssets([],
                                          [
                                              AssetId(chain: .bitcoin).identifier,
                                              AssetId(chain: .smartChain).identifier,
                                          ])],
            ).fetch(db)

            #expect(assets.count == 2)
        }
    }

    @Test func testSearch() throws {
        let db = DB.mock(wallets: [wallet], assets: walletAssets, balances: walletBalances)
        let searchStore = SearchStore(db: db)

        let query = "usdt ethereum"
        try searchStore.add(type: .asset, query: query, ids: [Asset.mock(id: .mock(chain: .ethereum, tokenId: "0xdAC17F958D2ee523a2206206994597C13D831ec7"), name: "Tether", symbol: "USDT", decimals: 6, type: .erc20).id.identifier])
        try searchStore.add(type: .asset, query: "T", ids: walletAssets.reversed().map(\.asset.id.identifier))

        try db.dbQueue.read { db in
            let btc = try AssetsQuery.mock(filters: [.search("btc", hasPriorityAssets: false)]).fetch(db)
            let bnb = try AssetsQuery.mock(filters: [.search("bNb", hasPriorityAssets: false)]).fetch(db)
            let tron = try AssetsQuery.mock(filters: [.search("0xdAC17F958D2ee523a2206206994597C13D831ec7", hasPriorityAssets: false)]).fetch(db)
            let searchAssets = try AssetsQuery.mock(searchBy: query).fetch(db)
            let prioritySearchAssets = try AssetsQuery.mock(filters: [.search("T", hasPriorityAssets: true)]).fetch(db)

            #expect(btc.count == 1)
            #expect(btc.first?.asset.symbol == "BTC")
            #expect(bnb.count == 1)
            #expect(bnb.first?.asset.name == "BNB")
            #expect(tron.count == 1)
            #expect(tron.first?.asset.symbol == "USDT")
            #expect(searchAssets.count == 1)
            #expect(searchAssets.first?.asset.symbol == "USDT")
            #expect(walletAssets.reversed().map(\.asset.id) == prioritySearchAssets.map(\.asset.id))
        }
    }

    @Test func typedSearchKeepsPinnedFirst() throws {
        let db = DB.mock(wallets: [wallet], assets: walletAssets, balances: walletBalances)
        let usdt = Asset.mock(id: .mock(chain: .ethereum, tokenId: "0xdAC17F958D2ee523a2206206994597C13D831ec7"), name: "Tether", symbol: "USDT", decimals: 6, type: .erc20).id
        try BalanceStore(db: db).setConfiguration(walletId: .mock(), assetIds: [usdt], configuration: .pinned(true))

        try db.dbQueue.read { db in
            let assets = try AssetsQuery.mock(searchBy: "t").fetch(db)

            #expect(assets.count > 1)
            #expect(assets.first?.asset.id == usdt)
        }
    }

    @Test func searchPrioritySortsHeldBalanceFirst() throws {
        let db = DB.mock(wallets: [wallet], assets: walletAssets, balances: walletBalances)
        let searchStore = SearchStore(db: db)
        let priceStore = PriceStore(db: db)

        try priceStore.saveRates([.mock()])

        let assets = walletAssets
        try priceStore.updatePrices(assets.map {
            .mock(assetId: $0.asset.id, price: 1, priceChangePercentage24h: 0)
        })

        let query = "usdt"
        try searchStore.add(type: .asset, query: query, ids: assets.map(\.asset.id.identifier))

        try db.dbQueue.read { db in
            let result = try AssetsQuery.mock(filters: [.search(query, hasPriorityAssets: true)]).fetch(db)

            #expect(result.map(\.asset.id) == assets.reversed().map(\.asset.id))
        }
    }

    @Test func searchNativeAssetByChainDoesNotMatchChainTokens() throws {
        let db = DB.mock(wallets: [.mock(accounts: [.mock(chain: .ton), .mock(chain: .base)])], assets: [
            .mock(asset: .mock(id: AssetId(chain: .ton), name: "Gram", symbol: "GRAM", decimals: 9, type: .native)),
            .mock(asset: .mock(id: AssetId(chain: .ton, tokenId: "abc"), name: "Tether", symbol: "USDT", decimals: 6, type: .jetton)),
            .mock(asset: .mock(id: AssetId(chain: .base), name: "Base ETH", symbol: "ETH", decimals: 18, type: .native)),
            .mock(asset: .mock(id: AssetId(chain: .base, tokenId: "0xtoken"), name: "Tether", symbol: "USDT", decimals: 6, type: .erc20)),
        ])

        try db.dbQueue.read { db in
            let ton = try AssetsQuery.mock(searchBy: "Ton").fetch(db)
            let base = try AssetsQuery.mock(searchBy: "Base").fetch(db)

            #expect(ton.map(\.asset.id) == [AssetId(chain: .ton)])
            #expect(base.map(\.asset.id) == [AssetId(chain: .base)])
        }
    }

    @Test func order() throws {
        let db = DB.mock(wallets: [wallet], assets: walletAssets, balances: walletBalances)
        let priceStore = PriceStore(db: db)
        let balanceStore = BalanceStore(db: db)

        try priceStore.saveRates([.mock()])

        try priceStore.updatePrices([.mock(assetId: AssetId(chain: .tron), price: 100, priceChangePercentage24h: 100)])

        try db.dbQueue.read { db in
            let assets = try AssetsQuery.mock().fetch(db)

            #expect(assets.first?.asset.id == AssetId(chain: .tron))
            #expect(assets.last?.asset.id == AssetId(chain: .ethereum, tokenId: "0xdAC17F958D2ee523a2206206994597C13D831ec7"))
        }

        try balanceStore.setConfiguration(walletId: .mock(), assetIds: [AssetId(chain: .bitcoin)], configuration: .pinned(true))
        try db.dbQueue.read { db in
            let assets = try AssetsQuery.mock().fetch(db)

            #expect(assets.first?.asset.id == AssetId(chain: .bitcoin))
        }
    }
}
