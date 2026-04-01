// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
@testable import PerpetualService
import Primitives

public actor PerpetualObserverMock: PerpetualObservable {
    public let chartService: any ChartStreamable = ChartObserverService()

    public private(set) var isConnected: Bool = false

    public init() {}

    public func setup(for _: Wallet) async {
        isConnected = true
    }

    public func disconnect() async {
        isConnected = false
    }

    public func update(for _: Wallet) async {}

    public func subscribe(_: HyperliquidSubscription) async throws {}
    public func unsubscribe(_: HyperliquidSubscription) async throws {}
}
