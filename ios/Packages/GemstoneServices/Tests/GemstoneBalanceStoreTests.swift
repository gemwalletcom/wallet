// Copyright (c). Gem Wallet. All rights reserved.

@testable import GemstoneServices
import Primitives
import PrimitivesTestKit
@testable import Store
import StoreTestKit
import Testing

struct GemstoneBalanceStoreTests {
    private let wallet = Wallet.mock(id: .multicoin(address: "0xtest"), accounts: [.mock(chain: .cosmos), .mock(chain: .ethereum)])
    private let ethereum = AssetId.mock(chain: .ethereum)
    private let cosmos = AssetId.mock(chain: .cosmos)

    @Test
    func addBalancesCarriesTheEnabledFlagCoreDecided() async throws {
        let db = DB.mock(wallets: [wallet])
        let adapter = GemstoneBalanceStore(store: .mock(db: db))
        let balanceStore = BalanceStore.mock(db: db)

        try await adapter.addBalances(walletId: wallet.id, assetIds: [ethereum], enabled: true)
        try await adapter.addBalances(walletId: wallet.id, assetIds: [cosmos], enabled: false)

        #expect(try balanceStore.getBalanceRecord(walletId: wallet.id, assetId: ethereum)?.isEnabled == true)
        #expect(try balanceStore.getBalanceRecord(walletId: wallet.id, assetId: cosmos)?.isEnabled == false)
    }

    @Test
    func addBalancesNeverOverwritesAStoredRow() async throws {
        let db = DB.mock(wallets: [wallet])
        let adapter = GemstoneBalanceStore(store: .mock(db: db))
        let balanceStore = BalanceStore.mock(db: db)
        try await adapter.addBalances(walletId: wallet.id, assetIds: [ethereum], enabled: true)

        try await adapter.addBalances(walletId: wallet.id, assetIds: [ethereum, cosmos], enabled: false)

        #expect(try balanceStore.getBalanceRecord(walletId: wallet.id, assetId: ethereum)?.isEnabled == true)
        #expect(try balanceStore.getBalanceRecord(walletId: wallet.id, assetId: cosmos)?.isEnabled == false)
    }
}
