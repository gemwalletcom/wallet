// Copyright (c). Gem Wallet. All rights reserved.

import protocol Gemstone.GemPriceAlertServiceProtocol
import GemstonePrimitivesTestKit
import PriceAlerts
import Primitives
import PrimitivesTestKit

public extension SetPriceAlertViewModel {
    @MainActor
    static func mock(
        service: any GemPriceAlertServiceProtocol = GemPriceAlertServiceMock(),
        onComplete: @escaping (String) -> Void = { _ in },
    ) -> SetPriceAlertViewModel {
        SetPriceAlertViewModel(
            walletId: .mock(),
            asset: .mock(),
            service: service,
            onComplete: onComplete,
        )
    }
}
