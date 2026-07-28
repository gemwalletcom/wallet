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

        #expect(WalletSearchTag.chain(.ethereum).includesPerpetuals == false)
        #expect(WalletSearchTag.chain(.ethereum).isList == false)
        #expect(WalletSearchTag.chain(.ethereum).isAll == false)
        #expect(WalletSearchTag.chain(.ethereum).chain == .ethereum)
        #expect(WalletSearchTag.all.chain == nil)
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
        #expect(WalletSearchTag.chain(.ethereum).apiTag == nil)
    }
}
