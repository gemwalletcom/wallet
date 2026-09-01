// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GemstonePrimitives
import Primitives
import struct Gemstone.GemRecentActivity
import protocol Gemstone.GemRecentActivityStore
import typealias Gemstone.WalletId

public final class GemstoneRecentActivityStore: GemRecentActivityStore, @unchecked Sendable {
    private let service: any RecentAssetsServiceable

    public init(service: any RecentAssetsServiceable) {
        self.service = service
    }

    public func add(activity: GemRecentActivity, walletId: Gemstone.WalletId) throws {
        try service.add(RecentActivityData(activity), walletId: Primitives.WalletId.from(id: walletId))
    }
}
