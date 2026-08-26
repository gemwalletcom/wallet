// Copyright (c). Gem Wallet. All rights reserved.

import GemstoneServicesTestKit
import GemstonePrimitives
import Primitives
import PrimitivesTestKit
@testable import Store
import StoreTestKit
import Testing
@testable import WalletService

struct WalletSetupServiceTests {
    @Test
    func setupMulticoinWallet() throws {
        let (db, balanceStore, walletStore, service) = setupService()
        let wallet = Wallet.mock(id: .multicoin(address: "0xtest"), type: .multicoin, accounts: [.mock(chain: .cosmos), .mock(chain: .ethereum)])

        try addAsset(db: db, chain: .cosmos)
        try addAsset(db: db, chain: .ethereum)
        try walletStore.addWallet(wallet)
        try service.setup(wallet: wallet)

        let isEnabled = try balanceStore.getBalanceRecord(walletId: wallet.id, assetId: AssetId(chain: .cosmos))?.isEnabled

        #expect(isEnabled == false)
    }

    @Test
    func setupSingleWallet() throws {
        let (db, balanceStore, walletStore, service) = setupService()
        let wallet = Wallet.mock(id: .single(chain: .cosmos, address: "0xtest"), type: .single, accounts: [.mock(chain: .cosmos)])

        try addAsset(db: db, chain: .cosmos)
        try walletStore.addWallet(wallet)
        try service.setup(wallet: wallet)

        let isEnabled = try balanceStore.getBalanceRecord(walletId: wallet.id, assetId: AssetId(chain: .cosmos))?.isEnabled

        #expect(isEnabled == true)
    }

    @Test
    func setupTronDefaultAssets() throws {
        let (db, balanceStore, walletStore, service) = setupService()
        let wallet = Wallet.mock(id: .multicoin(address: "0xtest"), type: .multicoin, accounts: [.mock(chain: .tron)])
        let assets = Chain.tron.defaultAssets

        try addAsset(db: db, asset: .mockTron())
        for asset in assets {
            try addAsset(db: db, asset: asset)
        }
        try walletStore.addWallet(wallet)
        try service.setup(wallet: wallet)

        for asset in assets {
            #expect(try balanceStore.getBalanceRecord(walletId: wallet.id, assetId: asset.id)?.isEnabled == true)
        }
    }

    @Test
    func setupTempoAssets() throws {
        let (db, balanceStore, walletStore, service) = setupService()
        let wallet = Wallet.mock(id: .single(chain: .tempo, address: "0xtest"), type: .single, accounts: [.mock(chain: .tempo)])
        let assets = Chain.tempo.defaultAssets

        try addAsset(db: db, asset: Asset(.tempo))
        for asset in assets {
            try addAsset(db: db, asset: asset)
        }
        try walletStore.addWallet(wallet)
        try service.setup(wallet: wallet)

        #expect(try balanceStore.getBalanceRecord(walletId: wallet.id, assetId: AssetId(chain: .tempo)) == nil)
        for asset in assets {
            #expect(try balanceStore.getBalanceRecord(walletId: wallet.id, assetId: asset.id)?.isEnabled == true)
        }
    }

    @Test
    func setupMulticoinTempoAssets() throws {
        let (db, balanceStore, walletStore, service) = setupService()
        let wallet = Wallet.mock(id: .multicoin(address: "0xtest"), type: .multicoin, accounts: [.mock(chain: .tempo)])
        let assets = Chain.tempo.defaultAssets

        try addAsset(db: db, asset: Asset(.tempo))
        for asset in assets {
            try addAsset(db: db, asset: asset)
        }
        try walletStore.addWallet(wallet)
        try service.setup(wallet: wallet)

        for asset in assets {
            #expect(try balanceStore.getBalanceRecord(walletId: wallet.id, assetId: asset.id)?.isEnabled == false)
        }
    }

    @Test
    func setupSingleSolanaAssets() throws {
        let (db, balanceStore, walletStore, service) = setupService()
        let wallet = Wallet.mock(id: .single(chain: .solana, address: "solana"), type: .single, accounts: [.mock(chain: .solana)])
        let assets = Chain.solana.defaultAssets

        try addAsset(db: db, asset: Asset(.solana))
        for asset in assets {
            try addAsset(db: db, asset: asset)
        }
        try walletStore.addWallet(wallet)
        try service.setup(wallet: wallet)

        #expect(try balanceStore.getBalanceRecord(walletId: wallet.id, assetId: AssetId(chain: .solana))?.isEnabled == true)
        for asset in assets {
            #expect(try balanceStore.getBalanceRecord(walletId: wallet.id, assetId: asset.id)?.isEnabled == true)
        }
    }

    @Test
    func setupMulticoinSolanaAssets() throws {
        let (db, balanceStore, walletStore, service) = setupService()
        let wallet = Wallet.mock(id: .multicoin(address: "solana"), type: .multicoin, accounts: [.mock(chain: .solana)])
        let assets = Chain.solana.defaultAssets

        try addAsset(db: db, asset: Asset(.solana))
        for asset in assets {
            try addAsset(db: db, asset: asset)
        }
        try walletStore.addWallet(wallet)
        try service.setup(wallet: wallet)

        #expect(try balanceStore.getBalanceRecord(walletId: wallet.id, assetId: AssetId(chain: .solana))?.isEnabled == true)
        for asset in assets {
            #expect(try balanceStore.getBalanceRecord(walletId: wallet.id, assetId: asset.id)?.isEnabled == false)
        }
    }

    private func setupService() -> (DB, BalanceStore, WalletStore, WalletSetupService) {
        let db = DB.mock()
        let balanceStore = BalanceStore.mock(db: db)
        let walletStore = WalletStore.mock(db: db)
        let service = WalletSetupService(balanceService: .mock(balanceStore: balanceStore))
        return (db, balanceStore, walletStore, service)
    }

    private func addAsset(db: DB, chain: Chain) throws {
        try addAsset(db: db, asset: .mock(id: AssetId(chain: chain)))
    }

    private func addAsset(db: DB, asset: Asset) throws {
        try db.dbQueue.write { db in
            try asset.record.insert(db)
        }
    }
}
