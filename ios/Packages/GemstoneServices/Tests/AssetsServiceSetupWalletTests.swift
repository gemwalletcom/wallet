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

    private func setupService(chains: [Chain]) -> (DB, BalanceStore, GemAssetsService) {
        let db = DB.mockWithChains(chains)
        let balanceStore = BalanceStore.mock(db: db)
        let service = GemAssetsService.mock(assetStore: AssetStore.mock(db: db), balanceStore: balanceStore)
        return (db, balanceStore, service)
    }
}
