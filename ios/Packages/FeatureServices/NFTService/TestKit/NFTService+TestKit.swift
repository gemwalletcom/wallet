// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GemAPITestKit
import NFTService
import StoreTestKit

public extension NFTService {
    static func mock() -> NFTService {
        NFTService(
            apiService: GemAPINFTServiceMock(),
            nftStore: .mock(),
        )
    }
}
