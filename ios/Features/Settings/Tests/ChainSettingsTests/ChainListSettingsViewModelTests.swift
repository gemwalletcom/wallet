// Copyright (c). Gem Wallet. All rights reserved.

import GemstonePrimitivesTestKit
@testable import Primitives
@testable import Settings
import Testing

@MainActor
struct ChainListSettingsViewModelTests {
    @Test
    func theNetworkListComesFromTheChainSettingsService() {
        let service = GemChainSettingsServiceMock()
        service.chainsValue = [Chain.ethereum.rawValue, Chain.bitcoin.rawValue]
        let model = ChainListSettingsViewModel(service: service)

        #expect(model.filterChains(for: "eth") == [.ethereum, .bitcoin])
        #expect(service.chainQueries == ["eth"])
    }
}
