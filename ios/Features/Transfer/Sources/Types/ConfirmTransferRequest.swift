// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GemstonePrimitives
import Primitives
import WalletConnector
import struct Gemstone.GemConfirmInput
import struct Gemstone.GemTransferData

public struct ConfirmTransferRequest: Sendable {
    public let wallet: Wallet
    public let data: GemTransferData
    public let simulation: SimulationResult?
    public let delegate: TransferDataCallback.ConfirmTransferDelegate?

    public init(
        wallet: Wallet,
        data: GemTransferData,
        simulation: SimulationResult? = nil,
        delegate: TransferDataCallback.ConfirmTransferDelegate? = nil,
    ) {
        self.wallet = wallet
        self.data = data
        self.simulation = simulation
        self.delegate = delegate
    }
}

public extension ConfirmTransferRequest {
    func confirmInput() throws -> GemConfirmInput {
        data.confirmInput(from: try wallet.account(for: data.chain))
    }
}
