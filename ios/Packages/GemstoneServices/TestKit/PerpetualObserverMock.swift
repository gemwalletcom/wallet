// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.GemPerpetualSubscription
@testable import GemstoneServices
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


    public func subscribe(_: GemPerpetualSubscription) async throws {}
    public func unsubscribe(_: GemPerpetualSubscription) async throws {}
}
