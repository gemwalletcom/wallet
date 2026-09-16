// Copyright (c). Gem Wallet. All rights reserved.

import Primitives
import Testing

struct WalletSearchTagTests {
    @Test
    func rules() {
        #expect(WalletSearchTag.list("stocks").isList)
        #expect(WalletSearchTag.all.isList == false)

        #expect(WalletSearchTag.all.isAll)
        #expect(WalletSearchTag.list("stocks").isAll == false)
    }
}
