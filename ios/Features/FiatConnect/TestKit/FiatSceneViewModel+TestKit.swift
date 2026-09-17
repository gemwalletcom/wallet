// Copyright (c). Gem Wallet. All rights reserved.

import FiatConnect
import GemstonePrimitivesTestKit
import Primitives
import PrimitivesTestKit

public extension FiatSceneViewModel {
    static func mock(
        assetAddress: AssetAddress = .mock(),
        type: FiatQuoteType = .buy,
        amount: Int? = nil,
    ) -> FiatSceneViewModel {
        FiatSceneViewModel(
            service: GemFiatQuoteServiceMock(),
            assetAddress: assetAddress,
            wallet: .mock(),
            type: type,
            amount: amount,
            locale: .US,
        )
    }
}
