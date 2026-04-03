// Copyright (c). Gem Wallet. All rights reserved.

import FiatService
import Foundation
import GemAPITestKit
import AssetsServiceTestKit
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
