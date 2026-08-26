// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import protocol Gemstone.GemStakeServiceProtocol
import GemstonePrimitivesTestKit
import GemstoneServices
import Store
import StoreTestKit

public extension StakeService {
    static func mock(
        store: StakeStore = .mock(),
        service: any GemStakeServiceProtocol = GemStakeServiceMock(),
    ) -> Self {
        StakeService(
            store: store,
            service: service,
        )
    }
}
