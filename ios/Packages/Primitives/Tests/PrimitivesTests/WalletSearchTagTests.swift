// Copyright (c). Gem Wallet. All rights reserved.

import Primitives
import Testing

struct WalletSearchTagTests {
    @Test
    func rules() {
        #expect(WalletSearchTag.all.includesPerpetuals)
        #expect(WalletSearchTag.list("stocks").includesPerpetuals)
        #expect(WalletSearchTag.filter(.stablecoins).includesPerpetuals == false)

        #expect(WalletSearchTag.list("stocks").isList)
        #expect(WalletSearchTag.all.isList == false)
        #expect(WalletSearchTag.filter(.stablecoins).isList == false)
    }

    @Test
    func searchKey() {
        #expect(WalletSearchTag.all.searchKey(query: "btc") == "btc")
        #expect(WalletSearchTag.list("stocks").searchKey(query: "") == "tag:stocks")
        #expect(WalletSearchTag.list("stocks").searchKey(query: "eth") == "eth")
        #expect(WalletSearchTag.filter(.trending).searchKey(query: "") == "tag:trending")
    }

    @Test
    func apiTag() {
        #expect(WalletSearchTag.all.apiTag == nil)
        #expect(WalletSearchTag.filter(.stablecoins).apiTag == "stablecoins")
        #expect(WalletSearchTag.list("stocks").apiTag == "stocks")
    }
}
