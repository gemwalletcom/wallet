// Copyright (c). Gem Wallet. All rights reserved.

import Gemstone
@testable import GemstonePrimitives
import Primitives
import PrimitivesTestKit
import Testing

final class TransferDataTypeMapTests {
    @Test
    func transferDataExtraMapPreservesTransactionType() throws {
        let extra = TransferDataExtra(to: "", transactionType: Primitives.TransactionType.tokenApproval)

        let mapped = try Primitives.TransactionType(extra.map().transactionType)
        #expect(mapped == Primitives.TransactionType.tokenApproval)
    }

    @Test
    func swapMapPreservesGasLimit() throws {
        let swapData = SwapData.mock(data: SwapQuoteData(
            to: "0x0000000000000000000000000000000000000001",
            dataType: .contract,
            value: "0",
            data: "0x",
            memo: nil,
            approval: .mock(),
            gasLimit: "500000",
        ))

        let mapped = try TransferDataType.swap(.mockEthereum(), .mockEthereum(), swapData).map()

        guard case let .swap(_, _, mappedSwapData) = mapped else {
            Issue.record("Expected swap input type")
            return
        }
        #expect(try Primitives.SwapData(mappedSwapData).data.gasLimit == "500000")
    }
}
