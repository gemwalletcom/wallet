// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import struct Gemstone.GemWalletConnectTransactionRequest
import GemstonePrimitives
import Primitives
import struct Gemstone.GemTransferData

public struct WCTransferData: Identifiable, Sendable {
    public let transferData: GemTransferData
    public let wallet: Wallet
    public let simulation: SimulationResult

    public init(transferData: GemTransferData, wallet: Wallet, simulation: SimulationResult) {
        self.transferData = transferData
        self.wallet = wallet
        self.simulation = simulation
    }

    public init(_ request: GemWalletConnectTransactionRequest) throws {
        try self.init(
            transferData: request.transfer,
            wallet: request.wallet.map(),
            simulation: SimulationResult(request.simulation),
        )
    }

    public var id: String {
        wallet.id.id
    }
}
