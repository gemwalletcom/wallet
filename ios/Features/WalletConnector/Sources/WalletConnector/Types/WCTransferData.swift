// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import struct Gemstone.GemTransferData
import struct Gemstone.GemWalletConnectTransactionRequest
import struct Gemstone.SimulationResult
import GemstonePrimitives
import Primitives

public struct WCTransferData: Identifiable, Sendable {
    public let transferData: GemTransferData
    public let wallet: Wallet
    public let simulation: SimulationResult

    public init(transferData: GemTransferData, wallet: Wallet, simulation: SimulationResult) {
        self.transferData = transferData
        self.wallet = wallet
        self.simulation = simulation
    }

    public init(_ request: GemWalletConnectTransactionRequest) {
        self.init(
            transferData: request.transfer,
            wallet: request.wallet.toPrimitives(),
            simulation: request.simulation,
        )
    }

    public var id: String {
        wallet.id.id
    }
}
