// Copyright (c). Gem Wallet. All rights reserved.

import Primitives
import PrimitivesTestKit
import Testing

struct Wallet_PrimitivesTests {
    @Test
    func canSign() {
        #expect(Wallet.mock(type: .multicoin).canSign == true)
        #expect(Wallet.mock(type: .view).canSign == false)
    }

}
