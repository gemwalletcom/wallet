// Copyright (c). Gem Wallet. All rights reserved.

@testable import GemstoneServices
import Primitives
import PrimitivesTestKit
@testable import Store
import StoreTestKit
import Testing

struct GemstoneAssetStoreTests {
    @Test
    func addBalancesCarriesTheEnabledFlagCoreDecided() async throws {
        let wallet = Wallet.mock(id: .multicoin(address: "0xtest"), accounts: [.mock(chain: .cosmos), .mock(chain: .ethereum)])
        let db = try DB.mockWithWallets([wallet])
        let balanceStore = BalanceStore.mock(db: db)
        let adapter = GemstoneAssetStore(assetStore: .mock(db: db), balanceStore: balanceStore)

        try await adapter.addBalances(walletId: wallet.id.id, assetIds: [AssetId(chain: .ethereum).identifier], enabled: true)
        try await adapter.addBalances(walletId: wallet.id.id, assetIds: [AssetId(chain: .cosmos).identifier], enabled: false)

        #expect(try balanceStore.getBalanceRecord(walletId: wallet.id, assetId: AssetId(chain: .ethereum))?.isEnabled == true)
        #expect(try balanceStore.getBalanceRecord(walletId: wallet.id, assetId: AssetId(chain: .cosmos))?.isEnabled == false)
    }

    @Test
    func addMissingBalancesNeverEnablesAndNeverOverwrites() async throws {
        let wallet = Wallet.mock(id: .multicoin(address: "0xtest"), accounts: [.mock(chain: .cosmos), .mock(chain: .ethereum)])
        let db = try DB.mockWithWallets([wallet])
        let balanceStore = BalanceStore.mock(db: db)
        let adapter = GemstoneAssetStore(assetStore: .mock(db: db), balanceStore: balanceStore)
        try await adapter.addBalances(walletId: wallet.id.id, assetIds: [AssetId(chain: .ethereum).identifier], enabled: true)

        try await adapter.addMissingBalances(
            walletId: wallet.id.id,
            assetIds: [AssetId(chain: .ethereum).identifier, AssetId(chain: .cosmos).identifier],
        )

        #expect(try balanceStore.getBalanceRecord(walletId: wallet.id, assetId: AssetId(chain: .ethereum))?.isEnabled == true)
        #expect(try balanceStore.getBalanceRecord(walletId: wallet.id, assetId: AssetId(chain: .cosmos))?.isEnabled == false)
    }
}
