// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import typealias Gemstone.BannerState
import struct Gemstone.GemBannerKey
import protocol Gemstone.GemBannerStore
import GemstonePrimitives
import Primitives
import Store

public final class GemstoneBannerStore: GemBannerStore, @unchecked Sendable {
    private let store: BannerStore

    public init(store: BannerStore) {
        self.store = store
    }

    public func getState(key: GemBannerKey) async throws -> Gemstone.BannerState? {
        try store.getBanner(id: key.identifier())
            .map { $0.state.map() }
    }

    public func setState(key: GemBannerKey, state: Gemstone.BannerState) async throws {
        let state = state.map()
        try store.addBanners([newBanner(key: key, state: state)])
        try store.updateState(key.identifier(), state: state)
    }

    public func addBanners(keys: [GemBannerKey], state: Gemstone.BannerState) async throws {
        let state = state.map()
        try store.addBanners(keys.map { try newBanner(key: $0, state: state) })
    }

    private func newBanner(key: GemBannerKey, state: Primitives.BannerState) throws -> NewBanner {
        try NewBanner(
            id: key.identifier(),
            walletId: key.walletId,
            assetId: key.assetId.map { try Primitives.AssetId(id: $0) },
            event: key.event.map(),
            state: state,
        )
    }
}
