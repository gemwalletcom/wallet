// Copyright (c). Gem Wallet. All rights reserved.

import BalanceServiceTestKit
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
    func setupTronUSDTByDefault() throws {
        let (db, balanceStore, walletStore, service) = setupService()
        let wallet = Wallet.mock(id: .multicoin(address: "0xtest"), type: .multicoin, accounts: [.mock(chain: .tron)])
        let asset = Asset.tronUSDT()

        try addAsset(db: db, asset: .mockTron())
        try addAsset(db: db, asset: asset)
        try walletStore.addWallet(wallet)
        try service.setup(wallet: wallet)

        let isEnabled = try balanceStore.getBalanceRecord(walletId: wallet.id, assetId: asset.id)?.isEnabled

        #expect(isEnabled == true)
    }

    @Test
    func setupTempoAssets() throws {
        let (db, balanceStore, walletStore, service) = setupService()
        let wallet = Wallet.mock(id: .single(chain: .tempo, address: "0xtest"), type: .single, accounts: [.mock(chain: .tempo)])

        try addAsset(db: db, asset: Asset(.tempo))
        try addAsset(db: db, asset: .tempoUSDC())
        try addAsset(db: db, asset: .tempoPathUSD())
        try walletStore.addWallet(wallet)
        try service.setup(wallet: wallet)

        #expect(try balanceStore.getBalanceRecord(walletId: wallet.id, assetId: AssetId(chain: .tempo)) == nil)
        #expect(try balanceStore.getBalanceRecord(walletId: wallet.id, assetId: Asset.tempoUSDC().id)?.isEnabled == true)
        #expect(try balanceStore.getBalanceRecord(walletId: wallet.id, assetId: Asset.tempoPathUSD().id)?.isEnabled == false)
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
