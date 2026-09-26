// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.GemAssetFilter
@testable import GemstoneServices
import Primitives
import PrimitivesTestKit
@testable import Store
import StoreTestKit
import Testing

struct GemstoneSwapStoreTests {
    @Test
    func assetIdsApplyTheFiltersTheyAreGivenAndLeadWithAPin() async throws {
        let wallet = Wallet.mock(id: .multicoin(address: "0xtest"), accounts: [.mock(chain: .bitcoin), .mock(chain: .ethereum), .mock(chain: .solana)])
        let db = DB.mock(wallets: [wallet])
        let assetStore = AssetStore.mock(db: db)
        let balanceStore = BalanceStore.mock(db: db)
        let store = GemstoneSwapStore(assetStore: assetStore, transactionStore: .mock(db: db), recentActivityStore: .mock(db: db))

        let pinned = AssetId(chain: .ethereum)
        let disabled = AssetId(chain: .solana)
        let unswappable = AssetId(chain: .bitcoin)
        try assetStore.add(assets: [
            .mock(asset: .mock(id: pinned), properties: .mock(isEnabled: true, isSwapable: true)),
            .mock(asset: .mock(id: disabled), properties: .mock(isEnabled: false, isSwapable: true)),
            .mock(asset: .mock(id: unswappable), properties: .mock(isEnabled: true, isSwapable: false)),
        ])
        try balanceStore.addMissingBalances(walletId: wallet.id, assetIds: [pinned, disabled, unswappable], isEnabled: true)
        _ = try balanceStore.setConfiguration(walletId: wallet.id, assetIds: [pinned], configuration: .pinned(true))

        let candidates = try await store.getAssetIds(walletId: wallet.id.id, filters: [.enabled, .swappable], limit: 10)

        #expect(candidates == [pinned.identifier])
    }

    @Test
    func assetIdsStopAtTheLimit() async throws {
        let wallet = Wallet.mock(id: .multicoin(address: "0xtest"), accounts: [.mock(chain: .bitcoin), .mock(chain: .ethereum), .mock(chain: .solana)])
        let db = DB.mock(wallets: [wallet])
        let assetStore = AssetStore.mock(db: db)
        let balanceStore = BalanceStore.mock(db: db)
        let store = GemstoneSwapStore(assetStore: assetStore, transactionStore: .mock(db: db), recentActivityStore: .mock(db: db))
        let assetIds = [AssetId(chain: .bitcoin), AssetId(chain: .ethereum), AssetId(chain: .solana)]

        try assetStore.add(assets: assetIds.map { .mock(asset: .mock(id: $0), properties: .mock(isEnabled: true, isSwapable: true)) })
        try balanceStore.addMissingBalances(walletId: wallet.id, assetIds: assetIds, isEnabled: true)

        let capped = try await store.getAssetIds(walletId: wallet.id.id, filters: [.enabled, .swappable], limit: 2)
        let all = try await store.getAssetIds(walletId: wallet.id.id, filters: [.enabled, .swappable], limit: 10)

        #expect(capped.count == 2)
        #expect(all.count == 3)
    }
}
