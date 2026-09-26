// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation
import Primitives
@testable import PrimitivesComponents
import PrimitivesComponentsTestKit
import PrimitivesTestKit
import Testing

@MainActor
struct NetworkFeeCustomViewModelTests {
    @Test
    func anEmptyFieldCannotBeConfirmed() {
        let model = NetworkFeeCustomViewModel.mock()

        #expect(model.isConfirmEnabled == false)
        #expect(model.value == nil || model.value?.isEmpty == false)
    }

    @Test
    func aBitcoinRateBelowTheMinimumIsRejectedWithItsOwnMessage() {
        let model = NetworkFeeCustomViewModel.mock(feeAsset: .mock(), unitType: .satVb, decimals: 0, baseFee: BigInt(200), baseTotal: BigInt(200), normalTotal: BigInt(400))
        model.input = "0"

        #expect(model.isConfirmEnabled == false)

        model.input = "1000000"
        #expect(model.isConfirmEnabled == false)
        #expect(model.errorText?.isNotEmpty == true)
    }

    @Test
    func aRateOverTheMaximumIsRejectedWithItsOwnMessage() {
        let model = NetworkFeeCustomViewModel.mock()
        model.input = "100"

        #expect(model.isConfirmEnabled == false)
        #expect(model.errorText?.isNotEmpty == true)
    }

    @Test
    func aReasonableRateConfirms() {
        let recorder = RateRecorder()
        let model = NetworkFeeCustomViewModel.mock(onSelect: { recorder.record($0) })
        model.input = "5"

        model.confirm()

        #expect(model.isConfirmEnabled)
        #expect(model.errorText == nil)
        #expect(recorder.rates == [BigInt(5_000_000_000)])
    }

    @Test
    func confirmingAnInvalidRateSendsNothing() {
        let recorder = RateRecorder()
        let model = NetworkFeeCustomViewModel.mock(onSelect: { recorder.record($0) })
        model.input = ""

        model.confirm()

        #expect(recorder.rates.isEmpty)
    }

    @Test
    func theFieldOnlyAcceptsWhatTheDecimalsAllow() {
        let model = NetworkFeeCustomViewModel.mock()

        #expect(model.sanitize("1.2345678901234") == "1.234567890")
        #expect(model.sanitize("abc") == "")
    }

    @Test
    func thePlaceholderShowsTheBaseTotal() {
        #expect(NetworkFeeCustomViewModel.mock().placeholder.isNotEmpty)
        #expect(NetworkFeeCustomViewModel.mock(baseTotal: nil).placeholder.isEmpty)
    }
}

private final class RateRecorder: @unchecked Sendable {
    private(set) var rates: [BigInt] = []

    func record(_ rate: BigInt) {
        rates.append(rate)
    }
}
