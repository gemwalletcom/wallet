// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.GemTransferAmount
import struct Gemstone.GemBalanceRequirement
import enum Gemstone.GemConfirmError
import struct Gemstone.GemConfirmMetadata
import GemstonePrimitivesTestKit
import BigInt
import GemstonePrimitives
import Preferences
import Primitives
import PrimitivesTestKit
import Testing
@testable import Transfer

struct TransactionInputViewModelTests {
    @Test
    func valueWithAmount() {
        let viewModel = TransactionInputViewModel(
            data: .mock(),
            fee: nil,
            metaData: nil,
            transferAmount: .success(GemTransferAmount(value: 200, networkFee: 1, isMaxAmount: false)),
            feeAsset: .mock(),
            currency: Currency.usd.rawValue,
        )

        #expect(viewModel.value == BigInt(200))
    }

    @Test
    func valueWithError() {
        let viewModel = TransactionInputViewModel(
            data: .mock(value: 100),
            fee: nil,
            metaData: nil,
            transferAmount: .failure(GemConfirmError.InsufficientBalance(asset: Asset.mock().map(), requirement: GemBalanceRequirement(required: 1, available: 0, shortfall: 1))),
            feeAsset: .mock(),
            currency: Currency.usd.rawValue,
        )

        #expect(viewModel.value == 100)
    }

    @Test
    func valueWithNilResult() {
        let viewModel = TransactionInputViewModel(
            data: .mock(),
            fee: nil,
            metaData: nil,
            transferAmount: nil,
            feeAsset: .mock(),
            currency: Currency.usd.rawValue,
        )

        #expect(viewModel.value == .zero)
    }

    @Test
    func testNetworkFeeText() {
        let viewModel = TransactionInputViewModel(
            data: .mock(),
            fee: .mock(fee: 1),
            metaData: nil,
            transferAmount: nil,
            feeAsset: .mock(),
            currency: Currency.usd.rawValue,
        )

        #expect(viewModel.networkFeeText == "0.00000001 BTC")
    }

    @Test
    func customFeeAsset() {
        let feeAsset = Asset.mockEthereumUSDT()
        let viewModel = TransactionInputViewModel(
            data: .mock(),
            fee: .mock(fee: 1_000_000, feeAsset: feeAsset.id.identifier),
            metaData: nil,
            transferAmount: nil,
            feeAsset: feeAsset,
            currency: Currency.usd.rawValue,
        )

        #expect(viewModel.networkFeeText == "1 USDT")
    }

    @Test
    func testNetworkFeeFiatText() {
        let assetId = AssetId.mock()
        let metaData = GemConfirmMetadata(
            assetBalance: .mock(assetId: assetId.identifier),
            feeAssetBalance: .mock(assetId: assetId.identifier),
            prices: [AssetPrice.mock(assetId: assetId, price: Price.mock().price, priceChangePercentage24h: 0).map()],
        )
        let viewModel = TransactionInputViewModel(
            data: .mock(),
            fee: .mock(fee: 1),
            metaData: metaData,
            transferAmount: nil,
            feeAsset: .mock(),
            currency: Currency.usd.rawValue,
        )

        #expect(viewModel.networkFeeFiatText == "$0.000000015")
    }

    @Test
    func nilFee() {
        let viewModel = TransactionInputViewModel(
            data: .mock(),
            fee: nil,
            metaData: nil,
            transferAmount: nil,
            feeAsset: .mock(),
            currency: Currency.usd.rawValue,
        )

        #expect(viewModel.networkFeeText == "-")
        #expect(viewModel.networkFeeFiatText == nil)
    }
}
