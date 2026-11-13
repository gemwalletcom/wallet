// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import struct Gemstone.GemLightningReceive
import enum Gemstone.GemLightningReceiveState
import protocol Gemstone.GemLightningServiceProtocol
import struct Gemstone.GemNumberFormat
import GemstonePrimitivesTestKit
import Primitives

public final class GemLightningServiceMock: GemLightningServiceProtocol, @unchecked Sendable {
    public var receive: GemLightningReceive
    public var receiveState: GemLightningReceiveState = .pending
    public var receiveError: (any Error)?

    public private(set) var receiveAmounts: [String] = []
    public private(set) var openEnabledCalls: [WalletId] = []
    public private(set) var syncCalls: [WalletId] = []
    public private(set) var lockAllCallsCount = 0

    public init(receive: GemLightningReceive = .mock(invoice: "lnbc1invoice", qr: "lightning:lnbc1invoice")) {
        self.receive = receive
    }

    public func lockAll() {
        lockAllCallsCount += 1
    }

    public func openEnabled(walletId: WalletId) async throws {
        openEnabledCalls.append(walletId)
    }

    public func receive(walletId _: WalletId, amount: String, format _: GemNumberFormat) async throws -> GemLightningReceive {
        receiveAmounts.append(amount)
        if let receiveError {
            throw receiveError
        }
        return receive
    }

    public func receiveState(walletId _: WalletId, invoice _: String) async throws -> GemLightningReceiveState {
        receiveState
    }

    public func sync(walletId: WalletId) async throws {
        syncCalls.append(walletId)
    }
}
