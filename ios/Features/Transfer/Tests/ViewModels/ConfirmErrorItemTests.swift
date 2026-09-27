// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.GemBalanceRequirement
import enum Gemstone.GemConfirmError
import struct Gemstone.GemConfirmFailure
import struct Gemstone.GemConfirmFee
import GemstonePrimitives
import GemstonePrimitivesTestKit
import Localization
@testable import Primitives
import PrimitivesTestKit
import Testing
@testable import Transfer
@testable import TransferTestKit

@MainActor
struct ConfirmErrorItemTests {
    @Test
    func loadFailureFillsTheList() {
        let error = GemConfirmError.Load(msg: "test error")
        let state = ConfirmTransferState.mock(screen: .mock(phase: .failed, failure: GemConfirmFailure(stage: .load, error: error)))
        let item = errorItem(state)

        guard case let .error(title, errorValue, onInfoAction) = item else {
            Issue.record("Expected .error")
            return
        }
        #expect(title == Localized.Errors.errorOccurred)
        #expect(errorValue as? GemConfirmError == error)
        #expect(onInfoAction == nil)
    }

    @Test
    func submitFailureStaysOutOfTheList() {
        let failure = GemConfirmFailure(stage: .execute, error: .Broadcast(hashes: [], msg: "rejected"))
        let state = ConfirmTransferState.mock(load: .mock(fee: .mock()), screen: .mock(phase: .failed, failure: failure))
        let item = errorItem(state)

        guard case .empty = item else {
            Issue.record("Expected .empty")
            return
        }
    }

    @Test
    func transferFailure() {
        let error = GemConfirmError.InsufficientBalance(asset: Asset.mock().toGem(), requirement: GemBalanceRequirement(required: 1, available: 0, shortfall: 1))
        let fee = GemConfirmFee.mock(amount: .error(error: error))
        let state = ConfirmTransferState.mock(load: .mock(fee: fee), screen: .mock(phase: .ready, failure: GemConfirmFailure(stage: .load, error: error)))
        let item = errorItem(state)

        guard case let .error(_, _, onInfoAction) = item else {
            Issue.record("Expected .error")
            return
        }
        #expect(onInfoAction != nil)
    }

    @Test
    func loaded() {
        let state = ConfirmTransferState.mock(load: .mock(fee: .mock()), screen: .mock(phase: .ready))
        let item = errorItem(state)
        guard case .empty = item else {
            Issue.record("Expected .empty")
            return
        }
    }

    private func errorItem(_ state: ConfirmTransferState) -> ConfirmTransferItemModel {
        let model = ConfirmTransferSceneViewModel.mock()
        model.state = state
        return model.itemModel(for: .error)
    }
}
