// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.GemSimulationPayloadRow
import PrimitivesComponents

public extension SimulationPayloadModel {
    static func mock(
        primaryFields: [GemSimulationPayloadRow] = [],
        secondaryFields: [GemSimulationPayloadRow] = [],
    ) -> SimulationPayloadModel {
        SimulationPayloadModel(
            primaryFields: primaryFields,
            secondaryFields: secondaryFields,
        )
    }
}
