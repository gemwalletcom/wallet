// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import struct Gemstone.GemWalletConnectMessageRequest
import struct Gemstone.SignMessage
import struct Gemstone.SimulationResult
import GemstonePrimitives
import GemstonePrimitivesTestKit
import Primitives
import PrimitivesTestKit

public extension GemWalletConnectMessageRequest {
    static func mock(
        chain: Chain = .ethereum,
        session: WalletConnectionSession = .mock(),
        wallet: Wallet = .mock(),
        message: SignMessage = .mock(),
        simulation: SimulationResult = .mock(),
        assets: [Asset] = [],
    ) -> GemWalletConnectMessageRequest {
        GemWalletConnectMessageRequest(
            sessionId: session.sessionId,
            chain: chain.rawValue,
            wallet: wallet.toGem(),
            account: (wallet.accounts.first ?? .mock(chain: chain)).toGem(),
            session: session.toGem(),
            simulation: simulation,
            message: message,
            assets: assets.map { $0.toGem() },
        )
    }
}
