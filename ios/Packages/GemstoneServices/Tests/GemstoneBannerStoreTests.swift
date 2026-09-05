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
        let db = DB.mockWithChains([.xrp, .cosmos])
        let store = BannerStore.mock(db: db)
        let adapter = GemstoneBannerStore(store: store)
        let wallet = Wallet.mock(id: .multicoin(address: "0xtest"), accounts: [.mock(chain: .xrp)])
        try WalletStore.mock(db: db).addWallet(wallet)
        let walletId = wallet.id
        let key = GemBannerKey(walletId: walletId.id, assetId: AssetId(chain: .xrp).identifier, event: Primitives.BannerEvent.accountActivation.map())

        try await adapter.addBanners(keys: [key], state: Primitives.BannerState.active.map())

        let row = try #require(try store.getBanner(id: key.identifier()))
        #expect(row.state == .active)
        #expect(row.walletId == walletId.id)
        #expect(row.assetId == AssetId(chain: .xrp))
        #expect(row.event == .accountActivation)
        #expect(try await adapter.getState(key: key) == Primitives.BannerState.active.map())
    }

    @Test
    func setStateCreatesTheRowWhenCoreHasNotSeededIt() async throws {
        let (store, adapter) = makeStore()
        let key = GemBannerKey(walletId: nil, assetId: AssetId(chain: .cosmos).identifier, event: Primitives.BannerEvent.stake.map())

        try await adapter.setState(key: key, state: Primitives.BannerState.cancelled.map())

        #expect(try store.getBanner(id: key.identifier())?.state == .cancelled)
    }

    @Test
    func addBannersLeavesAnExistingStateAlone() async throws {
        let (store, adapter) = makeStore()
        let key = GemBannerKey(walletId: nil, assetId: AssetId(chain: .cosmos).identifier, event: Primitives.BannerEvent.stake.map())
        try await adapter.setState(key: key, state: Primitives.BannerState.cancelled.map())

        try await adapter.addBanners(keys: [key], state: Primitives.BannerState.active.map())

        #expect(try store.getBanner(id: key.identifier())?.state == .cancelled)
    }

    private func makeStore() -> (BannerStore, GemstoneBannerStore) {
        let store = BannerStore.mock(db: .mockWithChains([.xrp, .cosmos]))
        return (store, GemstoneBannerStore(store: store))
    }
}
