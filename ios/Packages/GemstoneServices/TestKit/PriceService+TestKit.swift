// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import class Gemstone.GemApiClient
import class Gemstone.GemPriceService
import NativeProviderService
import GemstoneServices
import Primitives
import PrimitivesTestKit
import Store
import StoreTestKit

public extension PriceService {
    static func mock(db: DB = .mock()) -> Self {
        PriceService(
            priceStore: .mock(db: db),
            service: GemPriceService.mock(db: db),
        )
    }
}

public extension GemPriceService {
    static func mock(db: DB = .mock()) -> GemPriceService {
        GemPriceService(
            api: GemApiClient(
                provider: NativeProvider(url: Constants.apiURL),
                baseUrl: Constants.apiURL.absoluteString,
            ),
            store: GemstonePriceStore(priceStore: .mock(db: db), fiatRateStore: .mock(db: db)),
        )
    }
}
