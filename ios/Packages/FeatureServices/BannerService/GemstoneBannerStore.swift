// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import typealias Gemstone.BannerState
import struct Gemstone.GemBannerKey
import protocol Gemstone.GemBannerStore
import func Gemstone.bannerIdentifier
import GemstonePrimitives
import Primitives
import Store

public final class GemstoneBannerStore: GemBannerStore, @unchecked Sendable {
    private let store: BannerStore

    public init(store: BannerStore) {
        self.store = store
    }

    public func getState(key: GemBannerKey) async throws -> Gemstone.BannerState? {
        try store.getBanner(id: bannerIdentifier(key: key))
            .map { try $0.state.json() }
    }

    public func setState(key: GemBannerKey, state: Gemstone.BannerState) async throws {
        try store.updateState(
            bannerIdentifier(key: key),
            state: Primitives.BannerState(state),
        )
    }
}
