// Copyright (c). Gem Wallet. All rights reserved.

import Primitives
import Testing

@testable import GemstonePrimitives

final class ChainTests {
    @Test
    func assetIsSwappable() {
        #expect(Chain.ethereum.isSwapSupported)
        #expect(Chain.smartChain.isSwapSupported)
    }

    @Test
    func transactionTimeoutSeconds() {
        #expect(Chain.ethereum.transactionTimeoutSeconds == 1440)
        #expect(Chain.solana.transactionTimeoutSeconds == 75)
    }
}
