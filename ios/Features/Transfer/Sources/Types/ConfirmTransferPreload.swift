// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives

public struct ConfirmTransferPreload: Sendable {
    public let metadata: TransferDataMetadata
    public let input: ConfirmTransferInput
    public let feeRates: [FeeRate]
    public let simulation: SimulationResult?

    public init(
        metadata: TransferDataMetadata,
        input: ConfirmTransferInput,
        feeRates: [FeeRate],
        simulation: SimulationResult? = nil,
    ) {
        self.metadata = metadata
        self.input = input
        self.feeRates = feeRates
        self.simulation = simulation
    }
}
