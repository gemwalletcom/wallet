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
    private let ethereum = AssetId.mockEthereum()
    private let cosmos = AssetId.mock(.cosmos)

    @Test
    func addBalancesCarriesTheEnabledFlagCoreDecided() async throws {
        let db = try DB.mockWithWallets([wallet])
        let adapter = GemstoneAssetStore.mock(db: db)
        let balanceStore = BalanceStore.mock(db: db)

        try await adapter.addBalances(walletId: wallet.id.id, assetIds: [ethereum.identifier], enabled: true)
        try await adapter.addBalances(walletId: wallet.id.id, assetIds: [cosmos.identifier], enabled: false)

        #expect(try balanceStore.getBalanceRecord(walletId: wallet.id, assetId: ethereum)?.isEnabled == true)
        #expect(try balanceStore.getBalanceRecord(walletId: wallet.id, assetId: cosmos)?.isEnabled == false)
    }

    @Test
    func addMissingBalancesNeverEnablesAndNeverOverwrites() async throws {
        let db = try DB.mockWithWallets([wallet])
        let adapter = GemstoneAssetStore.mock(db: db)
        let balanceStore = BalanceStore.mock(db: db)
        try await adapter.addBalances(walletId: wallet.id.id, assetIds: [ethereum.identifier], enabled: true)

        try await adapter.addMissingBalances(walletId: wallet.id.id, assetIds: [ethereum.identifier, cosmos.identifier])

        #expect(try balanceStore.getBalanceRecord(walletId: wallet.id, assetId: ethereum)?.isEnabled == true)
        #expect(try balanceStore.getBalanceRecord(walletId: wallet.id, assetId: cosmos)?.isEnabled == false)
    }

    @Test
    func walletAssetsKeepOnlyWhatTheFiltersAllow() async throws {
        let db = try DB.mockWithWallets([wallet])
        let adapter = GemstoneAssetStore.mock(db: db)
        let balanceStore = BalanceStore.mock(db: db)
        try balanceStore.addBalance(assetIds: [ethereum, cosmos], isEnabled: true, for: wallet.id)
        try balanceStore.updateBalances([.mock(assetId: ethereum, available: 5)], for: wallet.id)

        #expect(try await adapter.getWalletAssets(walletId: wallet.id.id, filters: [.hasBalance]).map(\.id) == [ethereum.identifier])
        #expect(try await adapter.getWalletAssets(walletId: wallet.id.id, filters: []).count == 2)
    }

    @Test
    func chainsOrAssetIdsFilterMapsBothSlots() {
        #expect(GemAssetFilter.chainsOrAssetIds(chains: ["ethereum"], assetIds: ["smartchain_0x123"]).map() == .chainsOrAssets(["ethereum"], ["smartchain_0x123"]))
    }
}
