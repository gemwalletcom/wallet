// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import struct Gemstone.GemBalanceRequirement
import enum Gemstone.GemConfirmError
import struct Gemstone.GemConfirmMetadata
import struct Gemstone.GemTransferAmount
import GemstonePrimitives
import GemstonePrimitivesTestKit
import Primitives
import PrimitivesTestKit
import Testing
@testable import Transfer
import TransferTestKit

struct TransactionInputViewModelTests {
    @Test
    func valueWithAmount() {
        let viewModel = TransactionInputViewModel.mock(transferAmount: .success(GemTransferAmount(value: 200, networkFee: 1, isMaxAmount: false)))

        #expect(viewModel.value == BigInt(200))
    }

    @Test
    func valueWithError() {
        let viewModel = TransactionInputViewModel.mock(
            data: .mock(value: 100),
            transferAmount: .failure(GemConfirmError.InsufficientBalance(asset: Asset.mock().toGem(), requirement: GemBalanceRequirement(required: 1, available: 0, shortfall: 1))),
        )

        #expect(viewModel.value == 100)
    }

    @Test
    func valueWithNilResult() {
        let viewModel = TransactionInputViewModel.mock()

        #expect(viewModel.value == .zero)
    }
}
