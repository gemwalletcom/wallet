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

    @Test
    func searchKey() {
        #expect(WalletSearchTag.all.searchKey(query: "btc") == "btc")
        #expect(WalletSearchTag.list("stocks").searchKey(query: "") == "tag:stocks")
        #expect(WalletSearchTag.list("stocks").searchKey(query: "eth") == "eth")
    }

    @Test
    func apiTag() {
        #expect(WalletSearchTag.all.apiTag == nil)
        #expect(WalletSearchTag.list("stocks").apiTag == "stocks")
    }
}
