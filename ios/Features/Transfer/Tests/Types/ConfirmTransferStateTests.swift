// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.GemConfirmLoad
import GemstonePrimitivesTestKit
import Testing
@testable import Transfer
import TransferTestKit

struct ConfirmTransferStateTests {
    @Test
    func theStateCarriesTheFeeOfItsLoad() {
        #expect(ConfirmTransferState(.mock(fee: nil), screen: .mock()).fee == nil)
        #expect(ConfirmTransferState(.mock(fee: .mock()), screen: .mock()).fee != nil)
    }
}
