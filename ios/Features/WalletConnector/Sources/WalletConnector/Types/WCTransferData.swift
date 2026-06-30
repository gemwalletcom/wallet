// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives

public struct WCTransferData: Identifiable, Sendable {
    public let transferData: TransferData
    public let wallet: Wallet
    public let simulation: SimulationResult

    public init(transferData: TransferData, wallet: Wallet, simulation: SimulationResult) {
        self.transferData = transferData
        self.wallet = wallet
        self.simulation = simulation
    }

    public var id: String {
        wallet.id.id
    }
}
