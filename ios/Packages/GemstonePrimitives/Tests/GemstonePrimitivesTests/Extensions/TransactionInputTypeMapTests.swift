// Copyright (c). Gem Wallet. All rights reserved.

import Gemstone
@testable import GemstonePrimitives
import GemstonePrimitivesTestKit
import Primitives
import PrimitivesTestKit
import Testing

final class TransactionInputTypeMapTests {
    @Test
    func swapConstructorPreservesGasLimit() {
        let swapData = SwapData.mock(data: .mock(approval: .mock(), gasLimit: "500000"))

        let mapped = TransactionInputType.swap(.mock(id: .mock(chain: .ethereum), name: "Ethereum", symbol: "ETH", decimals: 18), .mock(id: .mock(chain: .ethereum), name: "Ethereum", symbol: "ETH", decimals: 18), swapData)

        guard case let .swap(_, _, mappedSwapData) = mapped else {
            Issue.record("Expected swap input type")
            return
        }
        #expect(mappedSwapData.data.gasLimit == "500000")
    }
}
