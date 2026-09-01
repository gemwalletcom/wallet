// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.GemConfirmMetadata
import GemstonePrimitives
import struct Gemstone.GemConfirmPreload
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

public extension ConfirmTransferPreload {
    init(_ preload: GemConfirmPreload) throws {
        self.init(
            metadata: preload.metadata,
            input: ConfirmTransferInput(
                confirmData: preload.confirmData,
                fee: try preload.confirmData.fee.map(),
                transferAmount: preload.amount.map(),
                feeAsset: preload.feeAsset.map(),
            ),
            feeRates: try preload.confirmData.feeRates.map { try $0.map() },
            simulation: try preload.confirmData.simulation.map { try Primitives.SimulationResult($0) },
        )
    }
}
