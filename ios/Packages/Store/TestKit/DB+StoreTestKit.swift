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

    static func mockAssetsWithPerpetualCollateralBalance() throws -> DB {
        let ethereum = Asset.mockEthereum()
        let bnb = Asset.mockBNB()
        let perpetual = Asset.hypercoreUSDC()
        let db = DB.mockAssets(assets: [
            .mock(asset: ethereum),
            .mock(asset: bnb),
            .mock(asset: perpetual),
        ])
        let balanceStore = BalanceStore(db: db)
        let fiatRateStore = FiatRateStore(db: db)
        let priceStore = PriceStore(db: db)

        try fiatRateStore.add([.mock()])
        try priceStore.updatePrices(
            prices: [
                .mock(assetId: ethereum.id, price: 100, priceChangePercentage24h: 0),
                .mock(assetId: bnb.id, price: 1000, priceChangePercentage24h: 0),
            ],
            currency: Currency.usd.rawValue,
        )
        try balanceStore.updateBalances(
            [
                .mockCoin(assetId: ethereum.id, available: 3),
                .mockCoin(assetId: bnb.id, available: 10),
                .mockPerpetual(assetId: perpetual.id, available: 50, reserved: 25),
            ],
            for: .mock(),
        )
        // hypercoreUSDC is an internal asset and is always isEnabled=false so it stays out of the asset list UI.
        try balanceStore.setIsEnabled(walletId: .mock(), assetIds: [bnb.id, perpetual.id], value: false)

        return db
    }

    static func mockAssetsWithEarnBalance() throws -> DB {
        let ethereum = Asset.mockEthereum()
        let db = DB.mockAssets(assets: [
            .mock(asset: ethereum),
        ])
        let balanceStore = BalanceStore(db: db)
        let fiatRateStore = FiatRateStore(db: db)
        let priceStore = PriceStore(db: db)

        try fiatRateStore.add([.mock()])
        try priceStore.updatePrices(
            prices: [
                .mock(assetId: ethereum.id, price: 110, priceChangePercentage24h: 10),
            ],
            currency: Currency.usd.rawValue,
        )
        try balanceStore.updateBalances(
            [
                .mockStake(assetId: ethereum.id, staked: 2),
                .mockEarn(assetId: ethereum.id, balance: 1),
            ],
            for: .mock(),
        )

        return db
    }
}
