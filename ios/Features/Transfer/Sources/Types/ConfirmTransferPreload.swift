// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.GemConfirmMetadata
import GemstonePrimitives
import struct Gemstone.GemConfirmPreload
import Foundation
import Primitives

public struct ConfirmTransferPreload: Sendable {
    public let metadata: GemConfirmMetadata
    public let input: ConfirmTransferInput
    public let simulation: SimulationResult?

    public init(
        metadata: GemConfirmMetadata,
        input: ConfirmTransferInput,
        simulation: SimulationResult? = nil,
    ) {
        self.metadata = metadata
        self.input = input
        self.simulation = simulation
    }
}

public extension ConfirmTransferPreload {
    init(_ preload: GemConfirmPreload) throws {
        self.init(
            metadata: preload.metadata,
            input: ConfirmTransferInput(
                confirmData: preload.confirmData,
                fee: preload.confirmData.fee,
                transferAmount: preload.amount.map(),
                feeAsset: preload.feeAsset.map(),
            ),
            simulation: try preload.confirmData.simulation.map { try Primitives.SimulationResult($0) },
        )
    }
}
