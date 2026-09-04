// Copyright (c). Gem Wallet. All rights reserved.

import Gemstone
@testable import GemstonePrimitives
import Primitives
import PrimitivesTestKit
import Testing

final class GemTransactionInputTypeMapTests {
    @Test
    func swapConstructorPreservesGasLimit() throws {
        let swapData = SwapData.mock(data: SwapQuoteData(
            to: "0x0000000000000000000000000000000000000001",
            dataType: .contract,
            value: "0",
            data: "0x",
            memo: nil,
            approval: .mock(),
            gasLimit: "500000",
        ))

        let mapped = GemTransactionInputType.swap(.mockEthereum(), .mockEthereum(), swapData)

        guard case let .swap(_, _, mappedSwapData) = mapped else {
            Issue.record("Expected swap input type")
            return
        }
        #expect(try Primitives.SwapData(mappedSwapData).data.gasLimit == "500000")
    }
}
