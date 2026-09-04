// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.GemBalanceRequirement
import Localization
@testable import Primitives
import PrimitivesTestKit
import GemstonePrimitives
import Testing
@testable import Transfer
import TransferTestKit

struct ConfirmErrorViewModelTests {
    @Test
    func error() {
        let error = AnyError("test error")
        let state = ConfirmTransferState.mock(transaction: .error(error))
        let model = ConfirmErrorViewModel(error: state.transactionError, onSelectListError: { _ in })

        guard case let .error(title, errorValue, _) = model.itemModel else {
            Issue.record("Expected .error")
            return
        }
        #expect(title == Localized.Errors.errorOccurred)
        #expect(errorValue.localizedDescription == error.localizedDescription)
    }

    @Test
    func transferFailure() {
        let input = ConfirmTransferInput.mock(transferAmount: .failure(.InsufficientBalance(asset: Asset.mock().map(), requirement: GemBalanceRequirement(required: 1, available: 0, shortfall: 1))))
        let state = ConfirmTransferState.mock(transaction: .data(input))
        let model = ConfirmErrorViewModel(error: state.transactionError, onSelectListError: { _ in })

        guard case .error = model.itemModel else {
            Issue.record("Expected .error")
            return
        }
    }

    @Test
    func loaded() {
        let state = ConfirmTransferState.mock(transaction: .data(.mock()))
        let model = ConfirmErrorViewModel(error: state.transactionError, onSelectListError: { _ in })
        guard case .empty = model.itemModel else {
            Issue.record("Expected .empty")
            return
        }
    }
}
