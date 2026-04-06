// Copyright (c). Gem Wallet. All rights reserved.

import AssetsServiceTestKit
import FiatService
import Foundation
import GemAPITestKit
import StoreTestKit

public extension FiatService {
    static func mock() -> FiatService {
        FiatService(
            apiService: GemAPIFiatServiceMock(),
            assetsService: .mock(),
            store: .mock(),
        )
    }
}
