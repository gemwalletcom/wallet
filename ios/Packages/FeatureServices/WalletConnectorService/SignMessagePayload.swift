// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import struct Gemstone.GemWalletConnectMessageRequest
import struct Gemstone.SignMessage
import GemstonePrimitives
import Primitives
import struct Gemstone.SimulationResult

public struct SignMessagePayload: Sendable {
    public let chain: Chain
    public let session: WalletConnectionSession
    public let wallet: Wallet
    public let message: SignMessage
    public let simulation: SimulationResult
    public let assets: [Asset]

    public init(
        chain: Chain,
        session: WalletConnectionSession,
        wallet: Wallet,
        message: SignMessage,
        simulation: SimulationResult,
        assets: [Asset],
    ) {
        self.chain = chain
        self.wallet = wallet
        self.session = session
        self.message = message
        self.simulation = simulation
        self.assets = assets
    }

    public init(_ request: GemWalletConnectMessageRequest) throws {
        try self.init(
            chain: Primitives.Chain(core: request.chain),
            session: request.session.map(),
            wallet: request.wallet.map(),
            message: request.message,
            simulation: request.simulation,
            assets: request.assets.map { $0.map() },
        )
    }
}

extension SignMessagePayload: Identifiable {
    public var id: String {
        session.id
    }
}
