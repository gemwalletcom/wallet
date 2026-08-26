// Copyright (c). Gem Wallet. All rights reserved.

import AssetsServiceTestKit
import FiatService
import Foundation
import GemstonePrimitivesTestKit
import StoreTestKit

public extension FiatService {
    static func mock() -> FiatService {
        FiatService(
            apiService: GemFiatServiceMock(),
            assetsService: .mock(),
            store: .mock(),
        )
    }
}
