// Copyright (c). Gem Wallet. All rights reserved.

import FiatService
import Foundation
import GemstonePrimitivesTestKit

public extension FiatService {
    static func mock() -> FiatService {
        FiatService(apiService: GemFiatServiceMock())
    }
}
