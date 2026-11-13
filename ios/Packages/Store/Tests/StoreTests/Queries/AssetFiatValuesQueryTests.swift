// Copyright (c). Gem Wallet. All rights reserved.

import Primitives
import PrimitivesTestKit
import Store
import StoreTestKit
import Testing

struct AssetFiatValuesQueryTests {
    @Test
    func walletBalanceWithPrice() throws {
        let usdt = AssetId(chain: .ethereum, tokenId: "0xdAC17F958D2ee523a2206206994597C13D831ec7")
        let db = DB.mock(
            wallets: [.mock(accounts: [.mock(chain: .bitcoin), .mock(chain: .smartChain), .mock(chain: .tron), .mock(chain: .ethereum)])],
            assets: [.bitcoin, .smartChain, .tron, .ethereum].map { .mock(asset: .mock(id: AssetId(chain: $0))) } + [.mock(asset: .mock(id: usdt, type: .erc20))],
            balances: [
                .mock(assetId: AssetId(chain: .smartChain), available: 1),
                .mock(assetId: AssetId(chain: .tron), available: 2),
                .mock(assetId: AssetId(chain: .ethereum), available: 3),
                .mock(assetId: usdt, available: 4),
            ],
        )
        try PriceStore(db: db).saveRatesAndPrices([FiatRate(symbol: .usd, rate: 1)], conversion: nil, prices: [])
        try PriceStore(db: db).updatePrices([.mock(assetId: AssetId(chain: .ethereum), price: 1100, priceChangePercentage24h: 10)])

        try db.dbPool.read { db in
            let result = try AssetFiatValuesQuery(walletId: .mock()).fetch(db)

            #expect(result.contains(AssetFiatValue(assetId: AssetId(chain: .ethereum), amount: 3, price: 1100, priceChangePercentage24h: 10)))
            #expect(result.filter { $0.price == 0 }.map(\.amount).sorted() == [0, 1, 2, 4])
        }
    }

    @Test
    func walletBalanceListsEnabledAssetsOnly() throws {
        let ethereum = Asset.mock(id: .mock(chain: .ethereum), name: "Ethereum", symbol: "ETH", decimals: 18)
        let bnb = Asset.mock(id: .mock(chain: .smartChain), name: "BNB", symbol: "BNB", decimals: 18)
        let perpetual = Asset.mock(id: .mock(chain: .hyperCore, tokenId: "perpetual::USDC"), name: "USDC", symbol: "USDC", decimals: 6, type: .perpetual)
        let db = DB.mock(
            wallets: [.mock(accounts: [ethereum, bnb, perpetual].map { .mock(chain: $0.chain) })],
            assets: [.mock(asset: ethereum), .mock(asset: bnb), .mock(asset: perpetual)],
            balances: [
                .mock(assetId: ethereum.id, available: 3),
                .mock(assetId: bnb.id, available: 10),
                .mock(assetId: perpetual.id, available: 50, reserved: 25),
            ],
        )
        try PriceStore(db: db).saveRatesAndPrices([.mock()], conversion: nil, prices: [])
        try PriceStore(db: db).updatePrices([
            .mock(assetId: ethereum.id, price: 100, priceChangePercentage24h: 0),
            .mock(assetId: bnb.id, price: 1000, priceChangePercentage24h: 0),
            .mock(assetId: perpetual.id, price: 0.92, priceChangePercentage24h: 0),
        ])
        try BalanceStore(db: db).setConfiguration(walletId: .mock(), assetIds: [bnb.id, perpetual.id], configuration: .disabled)

        try db.dbPool.read { db in
            let result = try AssetFiatValuesQuery(walletId: .mock()).fetch(db)

            #expect(result == [AssetFiatValue(assetId: ethereum.id, amount: 3, price: 100, priceChangePercentage24h: 0)])
        }
    }

    @Test
    func perpetualWalletBalanceCarriesTheStoredCollateral() throws {
        let ethereum = Asset.mock(id: .mock(chain: .ethereum), name: "Ethereum", symbol: "ETH", decimals: 18)
        let bnb = Asset.mock(id: .mock(chain: .smartChain), name: "BNB", symbol: "BNB", decimals: 18)
        let perpetual = Asset.mock(id: .mock(chain: .hyperCore, tokenId: "perpetual::USDC"), name: "USDC", symbol: "USDC", decimals: 6, type: .perpetual)
        let db = DB.mock(
            wallets: [.mock(accounts: [ethereum, bnb, perpetual].map { .mock(chain: $0.chain) })],
            assets: [.mock(asset: ethereum), .mock(asset: bnb), .mock(asset: perpetual)],
            balances: [
                .mock(assetId: ethereum.id, available: 3),
                .mock(assetId: bnb.id, available: 10),
                .mock(assetId: perpetual.id, available: 50, reserved: 25),
            ],
        )
        try PriceStore(db: db).saveRatesAndPrices([.mock()], conversion: nil, prices: [])
        try PriceStore(db: db).updatePrices([
            .mock(assetId: ethereum.id, price: 100, priceChangePercentage24h: 0),
            .mock(assetId: bnb.id, price: 1000, priceChangePercentage24h: 0),
            .mock(assetId: perpetual.id, price: 0.92, priceChangePercentage24h: 0),
        ])
        try BalanceStore(db: db).setConfiguration(walletId: .mock(), assetIds: [bnb.id, perpetual.id], configuration: .disabled)

        try db.dbPool.read { db in
            let result = try PerpetualWalletBalanceQuery(walletId: .mock(), assetId: perpetual.id).fetch(db)

            #expect(result?.balance.available == 50)
            #expect(result?.balance.reserved == 25)
            #expect(result?.price == 0.92, "the collateral carries the stored price, converted like every other asset")
        }
    }
}
