// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import struct Gemstone.GemSwapSession
import GemstonePrimitives
import GemstoneServicesTestKit
import Primitives
import PrimitivesTestKit
@testable import Swap

extension SwapButtonViewModel {
    static func mock(
        session: GemSwapSession = .mock(),
        value: BigInt = 1,
        availableBalance: BigInt = 2,
        fromAsset: AssetData? = .mock(),
    ) -> SwapButtonViewModel {
        SwapButtonViewModel(
            state: session.viewState(value: value, availableBalance: availableBalance, payAsset: nil),
            fromAsset: fromAsset,
            onAction: {},
        )
    }
}
