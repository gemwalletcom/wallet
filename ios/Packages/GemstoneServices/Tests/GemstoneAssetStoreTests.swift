// Copyright (c). Gem Wallet. All rights reserved.

import GemstonePrimitives
@testable import GemstoneServices
import Primitives
import PrimitivesTestKit
@testable import Store
import StoreTestKit
import Testing

struct GemstoneAssetStoreTests {
    @Test
    func addBalancesCarriesTheEnabledFlagCoreDecided() async throws {
        let (walletId, balanceStore, adapter) = try makeStore()

        try await adapter.addBalances(walletId: walletId.id, assetIds: [AssetId(chain: .ethereum).identifier], enabled: true)
        try await adapter.addBalances(walletId: walletId.id, assetIds: [AssetId(chain: .cosmos).identifier], enabled: false)

        #expect(try balanceStore.getBalanceRecord(walletId: walletId, assetId: AssetId(chain: .ethereum))?.isEnabled == true)
        #expect(try balanceStore.getBalanceRecord(walletId: walletId, assetId: AssetId(chain: .cosmos))?.isEnabled == false)
    }

    @Test
    func addMissingBalancesNeverEnablesAndNeverOverwrites() async throws {
        let (walletId, balanceStore, adapter) = try makeStore()
        try await adapter.addBalances(walletId: walletId.id, assetIds: [AssetId(chain: .ethereum).identifier], enabled: true)

        try await adapter.addMissingBalances(
            walletId: walletId.id,
            assetIds: [AssetId(chain: .ethereum).identifier, AssetId(chain: .cosmos).identifier],
        )

        #expect(try balanceStore.getBalanceRecord(walletId: walletId, assetId: AssetId(chain: .ethereum))?.isEnabled == true)
        #expect(try balanceStore.getBalanceRecord(walletId: walletId, assetId: AssetId(chain: .cosmos))?.isEnabled == false)
    }

    private func makeStore() throws -> (WalletId, BalanceStore, GemstoneAssetStore) {
        let db = DB.mockWithChains([.cosmos, .ethereum])
        let wallet = Wallet.mock(id: .multicoin(address: "0xtest"), accounts: [.mock(chain: .cosmos), .mock(chain: .ethereum)])
        try WalletStore.mock(db: db).addWallet(wallet)
        let balanceStore = BalanceStore.mock(db: db)
        return (wallet.id, balanceStore, GemstoneAssetStore(assetStore: AssetStore.mock(db: db), balanceStore: balanceStore))
    }
}
