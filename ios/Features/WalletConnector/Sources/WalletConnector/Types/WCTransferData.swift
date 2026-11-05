// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import struct Gemstone.GemTransferData
import struct Gemstone.GemWalletConnectTransactionRequest
import struct Gemstone.SimulationResult
import GemstonePrimitives
import Primitives

public struct WCTransferData: Sendable {
    public let requestId: String
    public let transferData: GemTransferData
    public let wallet: Wallet
    public let simulation: SimulationResult

    public init(requestId: String, transferData: GemTransferData, wallet: Wallet, simulation: SimulationResult) {
        self.requestId = requestId
        self.transferData = transferData
        self.wallet = wallet
        self.simulation = simulation
    }

    public init(_ request: GemWalletConnectTransactionRequest) {
        self.init(
            requestId: request.requestId,
            transferData: request.transfer,
            wallet: request.wallet.toPrimitives(),
            simulation: request.simulation,
        )
    }
}
