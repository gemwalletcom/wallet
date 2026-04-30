// Copyright (c). Gem Wallet. All rights reserved.

@testable import GemstonePrimitives
import Primitives
import Testing

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
