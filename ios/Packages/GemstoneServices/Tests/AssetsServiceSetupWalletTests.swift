// Copyright (c). Gem Wallet. All rights reserved.

import class Gemstone.GemAssetsService
import GemstonePrimitives
import GemstoneServicesTestKit
import Primitives
import PrimitivesTestKit
@testable import Store
import StoreTestKit
import Testing

struct AssetsServiceSetupWalletTests {
    @Test
    func setupMulticoinWallet() async throws {
        let (db, balanceStore, service) = setupService(chains: [.cosmos, .ethereum])
        let wallet = Wallet.mock(id: .multicoin(address: "0xtest"), type: .multicoin, accounts: [.mock(chain: .cosmos), .mock(chain: .ethereum)])
        try WalletStore.mock(db: db).addWallet(wallet)

        try await service.setupWallet(wallet: wallet.json())

        #expect(try balanceStore.getBalanceRecord(walletId: wallet.id, assetId: AssetId(chain: .cosmos))?.isEnabled == false)
        #expect(try balanceStore.getBalanceRecord(walletId: wallet.id, assetId: AssetId(chain: .ethereum))?.isEnabled == true)
    }

    @Test
    func setupSingleWallet() async throws {
        let (db, balanceStore, service) = setupService(chains: [.cosmos])
        let wallet = Wallet.mock(id: .single(chain: .cosmos, address: "0xtest"), type: .single, accounts: [.mock(chain: .cosmos)])
        try WalletStore.mock(db: db).addWallet(wallet)

        try await service.setupWallet(wallet: wallet.json())

        #expect(try balanceStore.getBalanceRecord(walletId: wallet.id, assetId: AssetId(chain: .cosmos))?.isEnabled == true)
    }

    @Test
    func setupTronDefaultAssets() async throws {
        let (db, balanceStore, service) = setupService(chains: [.tron])
        let wallet = Wallet.mock(id: .multicoin(address: "0xtest"), type: .multicoin, accounts: [.mock(chain: .tron)])
        try WalletStore.mock(db: db).addWallet(wallet)
        let assets = Chain.tron.defaultAssets

        for asset in assets {
            try addAsset(db: db, asset: asset)
        }
        try await service.setupWallet(wallet: wallet.json())

        for asset in assets {
            #expect(try balanceStore.getBalanceRecord(walletId: wallet.id, assetId: asset.id)?.isEnabled == true)
        }
    }

    @Test
    func setupTempoAssets() async throws {
        let (db, balanceStore, service) = setupService(chains: [.tempo])
        let wallet = Wallet.mock(id: .single(chain: .tempo, address: "0xtest"), type: .single, accounts: [.mock(chain: .tempo)])
        try WalletStore.mock(db: db).addWallet(wallet)
        let assets = Chain.tempo.defaultAssets

        for asset in assets {
            try addAsset(db: db, asset: asset)
        }
        try await service.setupWallet(wallet: wallet.json())

        #expect(try balanceStore.getBalanceRecord(walletId: wallet.id, assetId: AssetId(chain: .tempo)) == nil)
        for asset in assets {
            #expect(try balanceStore.getBalanceRecord(walletId: wallet.id, assetId: asset.id)?.isEnabled == true)
        }
    }

    private func setupService(chains: [Chain]) -> (DB, BalanceStore, GemAssetsService) {
        let db = DB.mockWithChains(chains)
        let balanceStore = BalanceStore.mock(db: db)
        let service = GemAssetsService.mock(assetStore: AssetStore.mock(db: db), balanceStore: balanceStore)
        return (db, balanceStore, service)
    }

    private func addAsset(db: DB, asset: Asset) throws {
        try AssetStore.mock(db: db).add(assets: [.mock(asset: asset)])
    }
}
