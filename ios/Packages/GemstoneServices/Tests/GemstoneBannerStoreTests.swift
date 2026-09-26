// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import struct Gemstone.GemBannerKey
import GemstonePrimitives
@testable import GemstoneServices
import Primitives
import PrimitivesTestKit
import Store
import StoreTestKit
import Testing

struct GemstoneBannerStoreTests {
    @Test
    func writesAndReadsBackTheStateCoreAsksFor() async throws {
        let wallet = Wallet.mock(id: .multicoin(address: "0xtest"), accounts: [.mock(chain: .xrp)])
        let store = BannerStore.mock(db: .mock(wallets: [wallet]))
        let adapter = GemstoneBannerStore(store: store)
        let walletId = wallet.id
        let key = GemBannerKey(walletId: walletId.id, assetId: AssetId(chain: .xrp).identifier, event: Primitives.BannerEvent.accountActivation.toGem())

        try await adapter.addBanners(keys: [key], state: Primitives.BannerState.active.toGem())

        let row = try #require(try store.getBanner(id: key.identifier()))
        #expect(row.state == .active)
        #expect(row.walletId == walletId.id)
        #expect(row.assetId == AssetId(chain: .xrp))
        #expect(row.event == .accountActivation)
        #expect(try await adapter.getState(key: key) == Primitives.BannerState.active.toGem())
    }

    @Test
    func setStateCreatesTheRowWhenCoreHasNotSeededIt() async throws {
        let store = BannerStore.mock(db: .mock(chains: [.cosmos]))
        let adapter = GemstoneBannerStore(store: store)
        let key = GemBannerKey(walletId: nil, assetId: AssetId(chain: .cosmos).identifier, event: Primitives.BannerEvent.stake.toGem())

        try await adapter.setState(key: key, state: Primitives.BannerState.cancelled.toGem())

        #expect(try store.getBanner(id: key.identifier())?.state == .cancelled)
    }

    @Test
    func addBannersLeavesAnExistingStateAlone() async throws {
        let store = BannerStore.mock(db: .mock(chains: [.cosmos]))
        let adapter = GemstoneBannerStore(store: store)
        let key = GemBannerKey(walletId: nil, assetId: AssetId(chain: .cosmos).identifier, event: Primitives.BannerEvent.stake.toGem())
        try await adapter.setState(key: key, state: Primitives.BannerState.cancelled.toGem())

        try await adapter.addBanners(keys: [key], state: Primitives.BannerState.active.toGem())

        #expect(try store.getBanner(id: key.identifier())?.state == .cancelled)
    }
}
