// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Formatters
import Foundation
import GemstoneFormatters
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
        let validator = AmountValidator.assetAmount(
            formatter: formatter,
            decimals: decimals,
            validators: [],
        )
        #expect(try validator.format("123.456") == BigInt(123_456_000))
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

    @Test
    func fiatAmountConvertsAndSucceeds() throws {
        let price = AssetPrice.mock(
            assetId: asset.id,
            price: 2,
            priceChangePercentage24h: .zero,
            updatedAt: .now,
        )
        let validator = AmountValidator.fiatAmount(
            formatter: formatter,
            converter: AssetValueConverter(),
            price: price,
            decimals: decimals,
            validators: [],
        )
        try validator.validate("10")
    }

    @Test
    func fiatAmountThrowsWhenPriceMissing() {
        let validator = AmountValidator.fiatAmount(
            formatter: formatter,
            converter: AssetValueConverter(),
            price: nil,
            decimals: decimals,
            validators: [],
        )
        #expect(throws: TransferError.invalidAmount) {
            try validator.validate("1.0")
        }
    }
}

private struct InvalidAmountValidator: ValueValidator {
    func validate(_: BigInt) throws {
        throw TransferError.invalidAmount
    }

    var id: String { "invalidAmount" }
}
