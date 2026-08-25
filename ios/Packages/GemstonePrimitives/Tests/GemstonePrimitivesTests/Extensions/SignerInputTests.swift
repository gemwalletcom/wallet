// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Gemstone
@testable import GemstonePrimitives
import Primitives
import PrimitivesTestKit
import Testing

final class SignerInputTests {
    @Test
    func transferDataExtraMapPreservesTransactionType() {
        let extra = TransferDataExtra(to: "", transactionType: Primitives.TransactionType.tokenApproval)

        #expect(extra.map().transactionType == Gemstone.TransactionType.tokenApproval)
    }

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

private extension SignerInput {
    static func mock(
        type: TransferDataType = .transfer(.mock()),
        asset: Asset = .mock(),
        value: BigInt = .zero,
        fee: Fee = .mock(),
        isMaxAmount: Bool = false,
        memo: String? = nil,
        senderAddress: String = "0x1234567890123456789012345678901234567890",
        destinationAddress: String = "0x1234567890123456789012345678901234567890",
        metadata: GemTransactionLoadMetadata = .none,
    ) -> SignerInput {
        SignerInput(
            type: type,
            asset: asset,
            value: value,
            fee: fee,
            isMaxAmount: isMaxAmount,
            memo: memo,
            senderAddress: senderAddress,
            destinationAddress: destinationAddress,
            metadata: metadata,
        )
    }
}
