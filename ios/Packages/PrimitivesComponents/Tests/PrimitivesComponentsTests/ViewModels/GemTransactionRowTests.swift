// Copyright (c). Gem Wallet. All rights reserved.

import Components
import struct Gemstone.GemTransactionRow
import GemstonePrimitives
import Primitives
import PrimitivesComponents
import PrimitivesComponentsTestKit
import PrimitivesTestKit
import Testing

final class GemTransactionRowTests {
    @Test
    func transactionTitle() {
        #expect(GemTransactionRow.mock(type: .transfer, state: .confirmed).titleTextValue.text == "Received")
        #expect(GemTransactionRow.mock(type: .transfer, state: .confirmed, direction: .outgoing).titleTextValue.text == "Sent")
        #expect(GemTransactionRow.mock(type: .transfer, state: .failed).titleTextValue.text == "Transfer")
        #expect(GemTransactionRow.mock(type: .swap).titleTextValue.text == "Swap")
    }
}
