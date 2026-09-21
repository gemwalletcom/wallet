// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import struct Gemstone.GemSwapRequest
import struct Gemstone.GemSwapSession
import GemstonePrimitives
import GemstoneServicesTestKit
import Primitives
import PrimitivesTestKit
@testable import Swap

extension SwapButtonViewModel {
    static func mock(
        session: GemSwapSession = .mock(),
        availableBalance: BigInt = BigInt(GemSwapRequest.mock.value),
        fromAsset: AssetData? = .mock(),
    ) -> SwapButtonViewModel {
        SwapButtonViewModel(
            state: session.viewState(availableBalance: availableBalance, payAsset: nil),
            fromAsset: fromAsset,
            onAction: {},
        )
    }
}
