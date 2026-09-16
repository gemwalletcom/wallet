// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation
import Primitives
import PrimitivesTestKit
@testable import PrimitivesComponents
import Testing

@MainActor
struct NetworkFeeCustomViewModelTests {
    private func model(
        chain: Chain = .ethereum,
        feeAsset: Asset = .mockEthereum(),
        unitType: FeeUnitType = .gwei,
        decimals: Int = 9,
        initialRate: BigInt? = nil,
        baseFee: BigInt? = BigInt(21000),
        baseTotal: BigInt? = BigInt(1_000_000_000),
        normalTotal: BigInt? = BigInt(2_000_000_000),
        onSelect: @escaping @MainActor (BigInt) -> Void = { _ in },
    ) -> NetworkFeeCustomViewModel {
        NetworkFeeCustomViewModel(
            chain: chain,
            feeAsset: feeAsset,
            feeAssetPrice: nil,
            currency: .usd,
            unitType: unitType,
            decimals: decimals,
            baseFee: baseFee,
            baseTotal: baseTotal,
            normalTotal: normalTotal,
            initialRate: initialRate,
            onSelect: onSelect,
        )
    }

    @Test
    func anInitialRateFillsTheField() {
        #expect(model(initialRate: BigInt(3_000_000_000)).input == "3")
        #expect(model(initialRate: nil).input.isEmpty)
    }

    @Test
    func anEmptyFieldCannotBeConfirmed() {
        let model = model()

        #expect(model.isConfirmEnabled == false)
        #expect(model.value == nil || model.value?.isEmpty == false)
    }

    @Test
    func aBitcoinRateBelowTheMinimumIsRejectedWithItsOwnMessage() {
        let model = model(chain: .bitcoin, feeAsset: .mock(), unitType: .satVb, decimals: 0, baseFee: BigInt(200), baseTotal: BigInt(200), normalTotal: BigInt(400))
        model.input = "0"

        #expect(model.isConfirmEnabled == false)

        model.input = "1000000"
        #expect(model.isConfirmEnabled == false)
        #expect(model.errorText?.isNotEmpty == true)
    }

    @Test
    func aRateOverTheMaximumIsRejectedWithItsOwnMessage() {
        let model = model()
        model.input = "100"

        #expect(model.isConfirmEnabled == false)
        #expect(model.errorText?.isNotEmpty == true)
    }

    @Test
    func aReasonableRateConfirms() {
        let recorder = RateRecorder()
        let model = model(onSelect: { recorder.record($0) })
        model.input = "5"

        model.confirm()

        #expect(model.isConfirmEnabled)
        #expect(model.errorText == nil)
        #expect(recorder.rates == [BigInt(5_000_000_000)])
    }

    @Test
    func confirmingAnInvalidRateSendsNothing() {
        let recorder = RateRecorder()
        let model = model(onSelect: { recorder.record($0) })
        model.input = ""

        model.confirm()

        #expect(recorder.rates.isEmpty)
    }

    @Test
    func theFieldOnlyAcceptsWhatTheDecimalsAllow() {
        let model = model()

        #expect(model.sanitize("1.2345678901234") == "1.234567890")
        #expect(model.sanitize("abc") == "")
    }

    @Test
    func thePlaceholderShowsTheBaseTotal() {
        #expect(model().placeholder.isNotEmpty)
        #expect(model(baseTotal: nil).placeholder.isEmpty)
    }
}

private final class RateRecorder: @unchecked Sendable {
    private(set) var rates: [BigInt] = []

    func record(_ rate: BigInt) {
        rates.append(rate)
    }
}
