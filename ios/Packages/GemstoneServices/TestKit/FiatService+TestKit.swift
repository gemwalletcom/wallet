// Copyright (c). Gem Wallet. All rights reserved.

import GemstoneServices
import Foundation
import GemstonePrimitivesTestKit

public extension FiatService {
    static func mock() -> FiatService {
        FiatService(service: GemFiatServiceMock())
    }
}
