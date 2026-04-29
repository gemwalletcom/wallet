// Copyright (c). Gem Wallet. All rights reserved.

import Primitives
import Testing

struct ChainTests {
    @Test
    func testShortAddress() {
        #expect(Chain.ethereum.shortAddress(address: "0x123") == "0x123")
        #expect(Chain.bitcoinCash.shortAddress(address: "bitcoincash:123") == "123")
    }

    @Test
    func testFullAddress() {
        #expect(Chain.ethereum.fullAddress(address: "0x123") == "0x123")
        #expect(Chain.bitcoinCash.fullAddress(address: "123") == "bitcoincash:123")
        #expect(Chain.bitcoinCash.fullAddress(address: "bitcoincash:123") == "bitcoincash:123")
    }
}
