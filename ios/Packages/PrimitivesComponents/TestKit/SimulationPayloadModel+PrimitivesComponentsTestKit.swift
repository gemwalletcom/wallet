// Copyright (c). Gem Wallet. All rights reserved.

import Primitives
import PrimitivesComponents

public extension SimulationPayloadModel {
    static func mock(
        chain: Chain = .ethereum,
        primaryFields: [SimulationPayloadField] = [],
        secondaryFields: [SimulationPayloadField] = [],
        addressNames: [ChainAddress: AddressName] = [:],
    ) -> SimulationPayloadModel {
        SimulationPayloadModel(
            chain: chain,
            primaryFields: primaryFields,
            secondaryFields: secondaryFields,
            addressNames: addressNames,
        )
    }
}
