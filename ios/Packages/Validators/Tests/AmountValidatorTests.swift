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
    func assetAmountSucceeds() throws {
        let validator = AmountValidator.assetAmount(
            formatter: formatter,
            decimals: decimals,
            validators: [PositiveValueValidator<BigInt>()],
        )
        try validator.validate("123.456")
    }

    @Test
    func assetAmountFailsMinimum() {
        let min = BigInt(1_000_000)
        let validator = AmountValidator.assetAmount(
            formatter: formatter,
            decimals: decimals,
            validators: [
                MinimumValueValidator<BigInt>(
                    minimumValue: min,
                    asset: asset,
                ),
            ],
        )

        #expect(throws: TransferError.minimumAmount(asset: asset, required: min)) {
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
            converter: ValueConverter(),
            price: price,
            decimals: decimals,
            validators: [PositiveValueValidator<BigInt>()],
        )
        try validator.validate("10")
    }

    @Test
    func fiatAmountThrowsWhenPriceMissing() {
        let validator = AmountValidator.fiatAmount(
            formatter: formatter,
            converter: ValueConverter(),
            price: nil,
            decimals: decimals,
            validators: [],
        )
        #expect(throws: TransferError.invalidAmount) {
            try validator.validate("1.0")
        }
    }
}
