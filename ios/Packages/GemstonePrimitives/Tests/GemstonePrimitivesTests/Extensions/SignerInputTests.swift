// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Gemstone
@testable import GemstonePrimitives
import Primitives
import PrimitivesTestKit
import Testing

final class SignerInputTests {
    @Test
    func mapPreservesSwapGasLimit() throws {
        let swapData = SwapData.mock(data: SwapQuoteData(
            to: "0x0000000000000000000000000000000000000001",
            dataType: .contract,
            value: "0",
            data: "0x",
            memo: nil,
            approval: .mock(),
            gasLimit: "500000",
        ))
        let input = SignerInput.mock(
            type: .swap(.mockEthereum(), .mockEthereum(), swapData),
            asset: .mockEthereum(),
            fee: .mock(gasLimit: BigInt(80000)),
        )

        let mapped = try input.map()

        guard case let .swap(_, _, mappedSwapData) = mapped.input.inputType else {
            Issue.record("Expected swap input type")
            return
        }
        #expect(mappedSwapData.data.gasLimit == "500000")
    }
}
