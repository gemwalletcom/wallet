// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Preferences
import Primitives
import PrimitivesTestKit
import Testing
@testable import Transfer
import Validators

struct TransactionInputViewModelTests {
    @Test
    func valueWithAmount() {
        let viewModel = TransactionInputViewModel(
            data: .mock(),
            transactionData: nil,
            metaData: nil,
            transferAmount: .success(TransferAmount(value: 200, networkFee: 1, useMaxAmount: false)),
        )

        #expect(viewModel.value == BigInt(200))
    }

    @Test
    func valueWithError() {
        let viewModel = TransactionInputViewModel(
            data: .mock(value: 100),
            transactionData: nil,
            metaData: nil,
            transferAmount: .failure(TransferAmountCalculatorError.insufficientBalance(.mock())),
        )

        #expect(viewModel.value == 100)
    }

    @Test
    func valueWithNilResult() {
        let viewModel = TransactionInputViewModel(
            data: .mock(),
            transactionData: nil,
            metaData: nil,
            transferAmount: nil,
        )

        #expect(viewModel.value == .zero)
    }

    @Test
    func testNetworkFeeText() {
        let viewModel = TransactionInputViewModel(
            data: .mock(),
            transactionData: .mock(),
            metaData: nil,
            transferAmount: nil,
        )

        #expect(viewModel.networkFeeText == "0.00000001 BTC")
    }

    @Test
    func testNetworkFeeFiatText() {
        let metaData = TransferDataMetadata(
            assetId: .mock(),
            feeAssetId: .mock(),
            assetBalance: .zero,
            assetFeeBalance: .zero,
            assetPrices: [.mock(): .mock()],
        )
        let viewModel = TransactionInputViewModel(
            data: .mock(),
            transactionData: .mock(),
            metaData: metaData,
            transferAmount: nil,
        )

        #expect(viewModel.networkFeeFiatText == "$0.000000015")
    }

    @Test
    func nilFee() {
        let viewModel = TransactionInputViewModel(
            data: .mock(),
            transactionData: nil,
            metaData: nil,
            transferAmount: nil,
        )

        #expect(viewModel.networkFeeText == "-")
        #expect(viewModel.networkFeeFiatText == nil)
    }
}
