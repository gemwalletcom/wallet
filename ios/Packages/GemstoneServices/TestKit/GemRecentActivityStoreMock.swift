// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
public import struct Gemstone.GemRecentActivity
public import protocol Gemstone.GemRecentActivityStore
public import typealias Gemstone.WalletId

public final class GemRecentActivityStoreMock: GemRecentActivityStore, @unchecked Sendable {
    private let lock = NSLock()
    private var added: [(activity: GemRecentActivity, walletId: WalletId)] = []

    public init() {}

    public var addedActivities: [GemRecentActivity] {
        lock.withLock { added.map(\.activity) }
    }

    public func add(activity: GemRecentActivity, walletId: WalletId) throws {
        lock.withLock { added.append((activity, walletId)) }
    }
}
