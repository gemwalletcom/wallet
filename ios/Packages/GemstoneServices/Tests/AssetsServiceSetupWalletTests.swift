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
        let (db, balanceStore, service) = setupService()
        let wallet = Wallet.mock(id: .multicoin(address: "0xtest"), type: .multicoin, accounts: [.mock(chain: .cosmos), .mock(chain: .ethereum)])

        try addAsset(db: db, chain: .cosmos)
        try addAsset(db: db, chain: .ethereum)
        try await service.setupWallet(wallet: wallet.json())

        #expect(try balanceStore.getBalanceRecord(walletId: wallet.id, assetId: AssetId(chain: .cosmos))?.isEnabled == false)
        #expect(try balanceStore.getBalanceRecord(walletId: wallet.id, assetId: AssetId(chain: .ethereum))?.isEnabled == true)
    }

    @Test
    func setupSingleWallet() async throws {
        let (db, balanceStore, service) = setupService()
        let wallet = Wallet.mock(id: .single(chain: .cosmos, address: "0xtest"), type: .single, accounts: [.mock(chain: .cosmos)])

        try addAsset(db: db, chain: .cosmos)
        try await service.setupWallet(wallet: wallet.json())

        #expect(try balanceStore.getBalanceRecord(walletId: wallet.id, assetId: AssetId(chain: .cosmos))?.isEnabled == true)
    }

    @Test
    func setupTronDefaultAssets() async throws {
        let (db, balanceStore, service) = setupService()
        let wallet = Wallet.mock(id: .multicoin(address: "0xtest"), type: .multicoin, accounts: [.mock(chain: .tron)])
        let assets = Chain.tron.defaultAssets

        try addAsset(db: db, asset: .mockTron())
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
        let (db, balanceStore, service) = setupService()
        let wallet = Wallet.mock(id: .single(chain: .tempo, address: "0xtest"), type: .single, accounts: [.mock(chain: .tempo)])
        let assets = Chain.tempo.defaultAssets

        try addAsset(db: db, asset: Asset(.tempo))
        for asset in assets {
            try addAsset(db: db, asset: asset)
        }
        try await service.setupWallet(wallet: wallet.json())

        #expect(try balanceStore.getBalanceRecord(walletId: wallet.id, assetId: AssetId(chain: .tempo)) == nil)
        for asset in assets {
            #expect(try balanceStore.getBalanceRecord(walletId: wallet.id, assetId: asset.id)?.isEnabled == true)
        }
    }

    @Test
    func setupFailsWithoutSeededAsset() async throws {
        let (_, _, service) = setupService()
        let wallet = Wallet.mock(id: .single(chain: .cosmos, address: "0xtest"), type: .single, accounts: [.mock(chain: .cosmos)])

        await #expect(throws: Error.self) {
            try await service.setupWallet(wallet: wallet.json())
        }
    }

    private func setupService() -> (DB, BalanceStore, GemAssetsService) {
        let db = DB.mock()
        let balanceStore = BalanceStore.mock(db: db)
        let service = GemAssetsService.mock(assetStore: AssetStore.mock(db: db), balanceStore: balanceStore)
        return (db, balanceStore, service)
    }

    private func addAsset(db: DB, chain: Chain) throws {
        try addAsset(db: db, asset: Asset.mock(id: AssetId(chain: chain)))
    }

    private func addAsset(db: DB, asset: Asset) throws {
        try AssetStore.mock(db: db).add(assets: [.mock(asset: asset)])
    }
}
