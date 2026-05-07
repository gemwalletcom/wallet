// Copyright (c). Gem Wallet. All rights reserved.

import GemstonePrimitives
import Primitives
import PrimitivesTestKit
import Testing

struct Chain_ChecksumTests {
    @Test
    func testChecksumAddress() {
        let bitcoinAddress = "bc1qr6f065nr70x4gl6ja9lm5wfj7xkhdv2sq04q23"
        let evmAddress = "0xd41fdb03ba84762dd66a0af1a6c8540ff1ba5dfb"
        let evmChecksumAddress = "0xD41FDb03Ba84762dD66a0af1a6C8540FF1ba5dfb"

        #expect(Chain.mock(.ethereum).checksumAddress(evmAddress) == evmChecksumAddress)
        #expect(Chain.mock(.smartChain).checksumAddress(evmAddress) == evmChecksumAddress)
        #expect(Chain.mock(.ethereum).checksumAddress(evmChecksumAddress) == evmChecksumAddress)
        #expect(Chain.mock(.bitcoin).checksumAddress(bitcoinAddress) == bitcoinAddress)
    }
}
