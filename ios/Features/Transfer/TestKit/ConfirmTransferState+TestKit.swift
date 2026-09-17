// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.GemConfirmLoad
import struct Gemstone.GemConfirmScreen
import GemstonePrimitivesTestKit
import Primitives
import PrimitivesTestKit
@testable import Transfer

extension ConfirmTransferState {
    static func mock(
        load: GemConfirmLoad? = nil,
        simulation: ConfirmSimulationState = .mock(),
        feeAsset: Asset = .mock(),
        screen: GemConfirmScreen = .mock(),
    ) -> ConfirmTransferState {
        ConfirmTransferState(feeAsset: feeAsset, load: load, simulation: simulation, screen: screen)
    }
}
