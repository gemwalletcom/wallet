// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.GemTransferAmount
import struct Gemstone.GemBalanceRequirement
import enum Gemstone.GemConfirmError
import struct Gemstone.GemConfirmMetadata
import GemstonePrimitivesTestKit
import BigInt
import GemstonePrimitives
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

    @Test
    func testNetworkFeeText() {
        let viewModel = TransactionInputViewModel.mock(fee: .mock(fee: 1))

        #expect(viewModel.networkFeeText == "0.00000001 BTC")
    }

    @Test
    func customFeeAsset() {
        let feeAsset = Asset.mockEthereumUSDT()
        let viewModel = TransactionInputViewModel.mock(
            fee: .mock(fee: 1_000_000, feeAsset: feeAsset.id.identifier),
            feeAsset: feeAsset,
        )

        #expect(viewModel.networkFeeText == "1 USDT")
    }

    @Test
    func testNetworkFeeFiatText() {
        let assetId = AssetId.mock()
        let metaData = GemConfirmMetadata.mock(
            assetId: assetId.identifier,
            prices: [AssetPrice.mock(assetId: assetId, price: Price.mock().price, priceChangePercentage24h: 0).toGem()],
        )
        let viewModel = TransactionInputViewModel.mock(
            fee: .mock(fee: 1),
            metaData: metaData,
        )

        #expect(viewModel.networkFeeFiatText == "$0.000000015")
    }

    @Test
    func nilFee() {
        let viewModel = TransactionInputViewModel.mock()

        #expect(viewModel.networkFeeText == "-")
        #expect(viewModel.networkFeeFiatText == nil)
    }
}
