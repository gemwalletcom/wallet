// Copyright (c). Gem Wallet. All rights reserved.

@testable import Blockchain
import Primitives
import Testing

struct EstimateFeeServiceTests {
    @Test
    func providerDoesNotOverrideBitcoinFees() {
        let provider = EstimateFeeService().provider(chain: .bitcoin)

        #expect(provider is EmptyService)
    }
}
