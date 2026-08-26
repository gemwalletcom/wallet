// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.GemPerpetualSubscription
import Primitives

public protocol PerpetualObservable: Actor {
    var chartService: any ChartStreamable { get }
    func setup(for wallet: Wallet) async
    func disconnect() async
    func update(for wallet: Wallet) async
    func subscribe(_ subscription: GemPerpetualSubscription) async throws
    func unsubscribe(_ subscription: GemPerpetualSubscription) async throws
}
