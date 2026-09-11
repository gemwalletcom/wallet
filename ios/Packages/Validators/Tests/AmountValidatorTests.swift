// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Formatters
import Foundation
import Primitives
import PrimitivesTestKit
import Testing
@testable import Validators

struct AmountValidatorTests {
    private let asset = Asset.mockEthereumUSDT()
    private let formatter = ValueFormatter(style: .full)
    private var decimals: Int {
        Int(asset.decimals)
    }

    @Test
    func assetAmountConverts() throws {
        let recorder = RecordingAmountValidator()
        let validator = AmountValidator.assetAmount(
            formatter: formatter,
            decimals: decimals,
            validators: [recorder],
        )
        try validator.validate("123.456")
        #expect(recorder.value == BigInt(123_456_000))
    }

    @Test
    func assetAmountPropagatesValidationFailure() {
        let validator = AmountValidator.assetAmount(
            formatter: formatter,
            decimals: decimals,
            validators: [InvalidAmountValidator()],
        )
        #expect(throws: TransferError.invalidAmount) {
            try validator.validate("0.5")
        }
    }
}

private struct InvalidAmountValidator: ValueValidator {
    func validate(_: BigInt) throws {
        throw TransferError.invalidAmount
    }

    var id: String { "invalidAmount" }
}

private final class RecordingAmountValidator: ValueValidator, @unchecked Sendable {
    var value: BigInt?

    func validate(_ value: BigInt) throws {
        self.value = value
    }

    var id: String { "recording" }
}
