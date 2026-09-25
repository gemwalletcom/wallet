// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives
import PrimitivesTestKit
import Store

public extension DB {
    static func mock() -> DB {
        DB(fileName: "\(UUID().uuidString).sqlite")
    }

    static func mockWithChains(_ chains: [Chain] = [.bitcoin]) -> DB {
        let db = Self.mock()
        let assetStore = AssetStore(db: db)
        try? assetStore.add(assets: chains.map { .mock(asset: .mock(id: $0.assetId)) })
        return db
    }

    static func mockWithWallets(_ wallets: [Wallet]) throws -> DB {
        let db = Self.mockWithChains(wallets.flatMap { $0.accounts.map(\.chain) }.asSet().asArray())
        let walletStore = WalletStore(db: db)
        for wallet in wallets {
            try walletStore.addWallet(wallet)
        }
        return db
    }

    static func mockAssets(assets: [AssetBasic] = .mock()) -> DB {
        let db = Self.mock()
        let assetStore = AssetStore(db: db)
        let balanceStore = BalanceStore(db: db)
        let walletStore = WalletStore(db: db)

        let existingChainIds = assets.filter { $0.asset.type == .native }.map(\.asset.chain).asSet()
        let allChains = assets.map(\.asset.chain).asSet()
        let missingChains = allChains.subtracting(existingChainIds)
        let chainAssets: [AssetBasic] = missingChains.map { .mock(asset: .mock(id: $0.assetId)) }

        try? assetStore.add(assets: assets + chainAssets)
        try? walletStore.addWallet(.mock(accounts: assets.map { Account.mock(chain: $0.asset.chain) }))
        try? balanceStore.addBalance(assets.map { AddBalance(assetId: $0.asset.id, isEnabled: true) }, for: .mock())
        try? balanceStore.updateBalances(.mock(assets: assets), for: .mock())

        return db
    }

    static func mockAssetsWithPrice(priceChangePercentage24h: Double) throws -> DB {
        let db = Self.mockAssets()
        try PriceStore(db: db).saveRates([FiatRate(symbol: .usd, rate: 1)])
        try PriceStore(db: db).updatePrices([
            .mock(assetId: AssetId(chain: .ethereum), price: 1100, priceChangePercentage24h: priceChangePercentage24h),
        ])
        return db
    }

    static func mockAssetsWithPerpetualCollateralBalance() throws -> DB {
        let ethereum = Asset.mock(id: .mock(chain: .ethereum), name: "Ethereum", symbol: "ETH", decimals: 18)
        let bnb = Asset.mock(id: .mock(chain: .smartChain), name: "BNB", symbol: "BNB", decimals: 18)
        let perpetual = Asset.mock(id: .mock(chain: .hyperCore, tokenId: "perpetual::USDC"), name: "USDC", symbol: "USDC", decimals: 6, type: .perpetual)
        let db = DB.mockAssets(assets: [
            .mock(asset: ethereum),
            .mock(asset: bnb),
            .mock(asset: perpetual),
        ])
        let balanceStore = BalanceStore(db: db)
        let priceStore = PriceStore(db: db)

        try priceStore.saveRates([.mock()])
        try priceStore.updatePrices([
            .mock(assetId: ethereum.id, price: 100, priceChangePercentage24h: 0),
            .mock(assetId: bnb.id, price: 1000, priceChangePercentage24h: 0),
            .mock(assetId: perpetual.id, price: 0.92, priceChangePercentage24h: 0),
        ])
        try balanceStore.updateBalances(
            [
                .mock(assetId: ethereum.id, available: 3),
                .mock(assetId: bnb.id, available: 10),
                .mock(assetId: perpetual.id, available: 50, reserved: 25),
            ],
            for: .mock(),
        )
        // hypercoreUSDC is an internal asset and is always isEnabled=false so it stays out of the asset list UI.
        try balanceStore.setConfiguration(walletId: .mock(), assetIds: [bnb.id, perpetual.id], configuration: .disabled)

        return db
    }
}
