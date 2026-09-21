// Copyright (c). Gem Wallet. All rights reserved.

@testable import GemstoneServices
import Primitives
import PrimitivesTestKit
@testable import Store
import StoreTestKit
import Testing

struct GemstoneSwapStoreTests {
    @Test
    func payCandidatesSkipTheDisabledAndTheUnswappableAndLeadWithAPin() async throws {
        let wallet = Wallet.mock(id: .multicoin(address: "0xtest"), accounts: [.mock(chain: .bitcoin), .mock(chain: .ethereum), .mock(chain: .solana)])
        let db = try DB.mockWithWallets([wallet])
        let assetStore = AssetStore.mock(db: db)
        let balanceStore = BalanceStore.mock(db: db)
        let store = GemstoneSwapStore(assetStore: assetStore, transactionStore: .mock(db: db), recentActivityStore: .mock(db: db))

        let pinned = AssetId(chain: .ethereum)
        let disabled = AssetId(chain: .solana)
        let unswappable = AssetId(chain: .bitcoin)
        try assetStore.add(assets: [
            .mock(asset: .mock(id: pinned), properties: .mock()),
            .mock(asset: .mock(id: disabled), properties: .swapCandidate(isEnabled: false)),
            .mock(asset: .mock(id: unswappable), properties: .swapCandidate(isSwapable: false)),
        ])
        try balanceStore.addMissingBalances(walletId: wallet.id, assetIds: [pinned, disabled, unswappable], isEnabled: true)
        _ = try balanceStore.setConfiguration(walletId: wallet.id, assetIds: [pinned], configuration: .pinned(true))

        let candidates = try await store.getPayAssetIds(walletId: wallet.id.id, limit: 10)

        #expect(candidates == [pinned.identifier], "a disabled asset and one no swapper takes are never a default pay asset")
    }

    @Test
    func payCandidatesStopAtTheLimit() async throws {
        let wallet = Wallet.mock(id: .multicoin(address: "0xtest"), accounts: [.mock(chain: .bitcoin), .mock(chain: .ethereum), .mock(chain: .solana)])
        let db = try DB.mockWithWallets([wallet])
        let assetStore = AssetStore.mock(db: db)
        let balanceStore = BalanceStore.mock(db: db)
        let store = GemstoneSwapStore(assetStore: assetStore, transactionStore: .mock(db: db), recentActivityStore: .mock(db: db))
        let assetIds = [AssetId(chain: .bitcoin), AssetId(chain: .ethereum), AssetId(chain: .solana)]

        try assetStore.add(assets: assetIds.map { .mock(asset: .mock(id: $0), properties: .mock()) })
        try balanceStore.addMissingBalances(walletId: wallet.id, assetIds: assetIds, isEnabled: true)

        let capped = try await store.getPayAssetIds(walletId: wallet.id.id, limit: 2)
        let all = try await store.getPayAssetIds(walletId: wallet.id.id, limit: 10)

        #expect(capped.count == 2)
        #expect(all.count == 3)
    }
}

private extension AssetProperties {
    static func swapCandidate(isEnabled: Bool = true, isSwapable: Bool = true) -> AssetProperties {
        AssetProperties(
            isEnabled: isEnabled,
            isBuyable: true,
            isSellable: true,
            isSwapable: isSwapable,
            isStakeable: false,
            stakingApr: nil,
            isEarnable: false,
            earnApr: nil,
            hasImage: true,
        )
    }
}
