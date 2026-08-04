// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import struct Gemstone.SignMessage
import Primitives

public struct SignMessagePayload: Sendable, Identifiable {
    public let id: String
    public let chain: Chain
    public let appMetadata: TransactionAppMetadata
    public let wallet: Wallet
    public let message: SignMessage
    public let simulation: SimulationResult
    public let payment: PaymentData?
    public let expiresAt: Date?

    public init(
        id: String,
        chain: Chain,
        appMetadata: TransactionAppMetadata,
        wallet: Wallet,
        message: SignMessage,
        simulation: SimulationResult,
        payment: PaymentData? = .none,
        expiresAt: Date? = .none,
    ) {
        self.id = id
        self.chain = chain
        self.appMetadata = appMetadata
        self.wallet = wallet
        self.message = message
        self.simulation = simulation
        self.payment = payment
        self.expiresAt = expiresAt
    }
}
