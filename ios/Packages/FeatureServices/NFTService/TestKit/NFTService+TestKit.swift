// Copyright (c). Gem Wallet. All rights reserved.

import GemAPI
import GemAPITestKit
import NFTService
import Store
import StoreTestKit

public extension NFTService {
    static func mock(
        apiService: any GemAPINFTService = GemAPINFTServiceMock(),
        nftStore: NFTStore = .mock(),
    ) -> NFTService {
        NFTService(
            apiService: apiService,
            nftStore: nftStore,
        )
    }
}
