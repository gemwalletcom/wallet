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

    @Test
    func addressValidation() {
        #expect(Chain.ethereum.isValidAddress("0x95222290DD7278Aa3Ddd389Cc1E1d165CC4BAfe5"))
        #expect(Chain.ton.isValidAddress(" \nUQAzoUpalAaXnVm5MoiYWRZguLFzY0KxFjLv3MkRq5BXz3VV\r "))
        #expect(!Chain.ethereum.isValidAddress("0x123"))
        #expect(Chain.cardano.isValidAddress("addr1q8043m5heeaydnvtmmkyuhe6qv5havvhsf0d26q3jygsspxlyfpyk6yqkw0yhtyvtr0flekj84u64az82cufmqn65zdsylzk23"))
        #expect(!Chain.cardano.isValidAddress("addr1q8043m5heeaydnvtmmkyuhe6qv5havvhsf0d26q3jygsspxlyfpyk6yqkw0yhtyvtr0flekj84u64az82cufmqn65zdsylzk2x"))
    }
}
