// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.GemConfirmMetadata
import Foundation
import Primitives

public struct ConfirmTransferPreload: Sendable {
    public let metadata: GemConfirmMetadata
    public let input: ConfirmTransferInput
    public let feeRates: [FeeRate]
    public let simulation: SimulationResult?

    public init(
        metadata: GemConfirmMetadata,
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
