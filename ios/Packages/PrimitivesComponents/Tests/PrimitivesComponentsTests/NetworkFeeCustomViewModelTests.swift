// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation
import enum Gemstone.GemConfirmFeeSelection
import Primitives
@testable import PrimitivesComponents
import PrimitivesComponentsTestKit
import PrimitivesTestKit
import Testing

@MainActor
struct NetworkFeeCustomViewModelTests {
    @Test
    func anEmptyFieldConfirmsTheSuggestedRate() {
        let recorder = SelectionRecorder()
        let model = NetworkFeeCustomViewModel.mock(onSelect: { recorder.record($0) })

        model.confirm()

        #expect(model.isConfirmEnabled)
        #expect(recorder.selections == [.custom(baseFee: nil, rate: nil)], "nothing typed, so the network rate is used")
    }

    @Test
    func aReasonableRateConfirms() {
        let recorder = SelectionRecorder()
        let model = NetworkFeeCustomViewModel.mock(onSelect: { recorder.record($0) })
        model.input = "5"

        model.confirm()

        #expect(model.isConfirmEnabled)
        #expect(model.rateField.errorText == nil)
        #expect(recorder.selections == [.custom(baseFee: nil, rate: BigInt(5_000_000_000))])
    }

    @Test
    func confirmingAnInvalidRateSendsNothing() {
        let recorder = SelectionRecorder()
        let model = NetworkFeeCustomViewModel.mock(onSelect: { recorder.record($0) })
        model.input = "100"

        model.confirm()

        #expect(recorder.selections.isEmpty)
    }

    @Test
    func theFieldOnlyAcceptsWhatTheDecimalsAllow() {
        let model = NetworkFeeCustomViewModel.mock()

        #expect(model.sanitize("1.2345678901234") == "1.234567890")
        #expect(model.sanitize("abc") == "")
    }

    @Test
    func thePlaceholderShowsTheNormalRate() {
        #expect(NetworkFeeCustomViewModel.mock().rateField.placeholder.isNotEmpty)
        #expect(NetworkFeeCustomViewModel.mock(normal: nil).rateField.placeholder.isEmpty)
        #expect(NetworkFeeCustomViewModel.mock().baseFeeField == nil, "a rate with no base fee is one field")
    }

    @Test
    func aBaseFeeChainTakesBothFieldsAndSendsThemTogether() {
        let recorder = SelectionRecorder()
        let model = NetworkFeeCustomViewModel.mock(
            selected: .eip1559(gasPrice: BigInt(24_000_000_000), priorityFee: BigInt(1_000_000_000)),
            normal: .eip1559(gasPrice: BigInt(24_000_000_000), priorityFee: BigInt(1_000_000_000)),
            networkBaseFee: BigInt(20_000_000_000),
            onSelect: { recorder.record($0) },
        )
        model.input = "5"
        #expect(model.isConfirmEnabled, "a tip alone rides the network's base fee")

        model.baseFeeInput = "19"
        #expect(model.baseFeeField?.errorText?.isNotEmpty == true, "a base fee under the network's is refused")
        #expect(model.isConfirmEnabled == false)

        model.baseFeeInput = "22"
        model.confirm()
        #expect(recorder.selections == [.custom(baseFee: BigInt(22_000_000_000), rate: BigInt(5_000_000_000))])
    }
}

private final class SelectionRecorder: @unchecked Sendable {
    private(set) var selections: [GemConfirmFeeSelection] = []

    func record(_ selection: GemConfirmFeeSelection) {
        selections.append(selection)
    }
}
