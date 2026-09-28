// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.GemAssetFilter
@testable import GemstoneServices
import GemstoneServicesTestKit
import Primitives
import PrimitivesTestKit
@testable import Store
import StoreTestKit
import Testing

struct GemstoneAssetStoreTests {
    private let wallet = Wallet.mock(id: .multicoin(address: "0xtest"), accounts: [.mock(chain: .cosmos), .mock(chain: .ethereum)])
    private let ethereum = AssetId.mock(chain: .ethereum)
    private let cosmos = AssetId.mock(chain: .cosmos)

    @Test
    func addBalancesCarriesTheEnabledFlagCoreDecided() async throws {
        let db = DB.mock(wallets: [wallet])
        let adapter = GemstoneAssetStore.mock(db: db)
        let balanceStore = BalanceStore.mock(db: db)

        try await adapter.addBalances(walletId: wallet.id, assetIds: [ethereum], enabled: true)
        try await adapter.addBalances(walletId: wallet.id, assetIds: [cosmos], enabled: false)

        #expect(try balanceStore.getBalanceRecord(walletId: wallet.id, assetId: ethereum)?.isEnabled == true)
        #expect(try balanceStore.getBalanceRecord(walletId: wallet.id, assetId: cosmos)?.isEnabled == false)
    }

    @Test
    func addMissingBalancesNeverEnablesAndNeverOverwrites() async throws {
        let db = DB.mock(wallets: [wallet])
        let adapter = GemstoneAssetStore.mock(db: db)
        let balanceStore = BalanceStore.mock(db: db)
        try await adapter.addBalances(walletId: wallet.id, assetIds: [ethereum], enabled: true)

        try await adapter.addMissingBalances(walletId: wallet.id, assetIds: [ethereum, cosmos])

        #expect(try balanceStore.getBalanceRecord(walletId: wallet.id, assetId: ethereum)?.isEnabled == true)
        #expect(try balanceStore.getBalanceRecord(walletId: wallet.id, assetId: cosmos)?.isEnabled == false)
    }

    @Test
    func walletAssetsKeepOnlyWhatTheFiltersAllow() async throws {
        let db = DB.mock(wallets: [wallet])
        let adapter = GemstoneAssetStore.mock(db: db)
        let balanceStore = BalanceStore.mock(db: db)
        try balanceStore.addBalance(assetIds: [ethereum, cosmos], isEnabled: true, for: wallet.id)
        try balanceStore.updateBalances([.mock(assetId: ethereum, available: 5)], for: wallet.id)

        #expect(try await adapter.getWalletAssets(walletId: wallet.id, filters: [.hasBalance]).map(\.id) == [ethereum])
        #expect(try await adapter.getWalletAssets(walletId: wallet.id, filters: []).count == 2)
    }

    @Test
    func chainsOrAssetIdsFilterMapsBothSlots() {
        #expect(GemAssetFilter.chainsOrAssetIds(chains: ["ethereum"], assetIds: [AssetId(chain: .smartChain, tokenId: "0x123")]).map() == .chainsOrAssets(["ethereum"], ["smartchain_0x123"]))
    }
}
