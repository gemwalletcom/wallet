// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
@testable import Primitives
import PrimitivesTestKit
import Testing
@testable import Transfer
import TransferTestKit
import Validators

struct ConfirmTransferInputProviderTests {
    @Test
    func loadReturnsInputWithRatesAndFee() async throws {
        let provider = ConfirmTransferInputProvider.mock(transaction: .success(
            TransferTransactionData(
                allRates: [FeeRate(priority: .normal, gasPriceType: .regular(gasPrice: 1))],
                transactionData: .mock(),
            ),
        ))

        let result = try await provider.load(request: .mock(), metadata: .mock(), selection: .preset(.normal))

        #expect(result.feeRates.count == 1)
        #expect(result.input.transactionData.fee.fee == 1)
    }

    @Test
    func loadUsesFeeAssetFromTransactionFee() async throws {
        let feeAsset = Asset.hypercoreUSDC()
        let feeAssetBalance = Balance.mock(available: 42)
        let feeAssetPrice = Price.mock(price: 1)
        let provider = ConfirmTransferInputProvider(
            transferTransactionProvider: TransferTransactionProviderMock(result: .success(
                TransferTransactionData(
                    allRates: [],
                    transactionData: .mock(feeAsset: feeAsset),
                ),
            )),
            feeAssetProvider: FeeAssetProviderMock(asset: feeAsset, balance: feeAssetBalance, price: feeAssetPrice),
        )

        let result = try await provider.load(request: .mock(), metadata: .mock(), selection: .preset(.normal))

        #expect(result.input.feeAsset == feeAsset)
        #expect(result.metadata.assetFeeBalance == feeAssetBalance)
        #expect(result.metadata.feeAssetId == feeAsset.id)
        #expect(result.metadata.feePrice == feeAssetPrice)
    }

    @Test
    func loadMapsPreloadFailureToInsufficientNetworkFee() async {
        let provider = ConfirmTransferInputProvider.mock(transaction: .failure(AnyError("network")))
        let metadata = TransferDataMetadata.mock(
            feeAssetId: AssetId(chain: .ethereum, tokenId: nil),
            assetFeeBalance: .mock(available: .zero),
        )

        await #expect(throws: TransferAmountCalculatorError.self) {
            try await provider.load(request: .mock(), metadata: metadata, selection: .preset(.normal))
        }
    }

    @Test
    func loadRethrowsPreloadFailureWhenFeeBalanceAvailable() async {
        let provider = ConfirmTransferInputProvider.mock(transaction: .failure(AnyError("network")))
        let metadata = TransferDataMetadata.mock(
            feeAssetId: AssetId(chain: .ethereum, tokenId: nil),
            assetFeeBalance: .mock(available: 1000),
        )

        await #expect(throws: AnyError.self) {
            try await provider.load(request: .mock(), metadata: metadata, selection: .preset(.normal))
        }
    }

    @Test
    func loadRethrowsPreloadFailureForTempoFeeAsset() async {
        let provider = ConfirmTransferInputProvider.mock(transaction: .failure(AnyError("network")))
        let metadata = TransferDataMetadata.mock(
            feeAssetId: Asset.tempoPathUSD().id,
            assetFeeBalance: .mock(available: .zero),
        )

        await #expect(throws: AnyError.self) {
            try await provider.load(request: .mock(), metadata: metadata, selection: .preset(.normal))
        }
    }

    @Test
    func loadRethrowsFeeAssetFailure() async {
        let provider = ConfirmTransferInputProvider(
            transferTransactionProvider: TransferTransactionProviderMock(result: .success(
                TransferTransactionData(allRates: [], transactionData: .mock()),
            )),
            feeAssetProvider: FeeAssetProviderMock(error: "fee asset"),
        )
        let metadata = TransferDataMetadata.mock(
            feeAssetId: AssetId(chain: .ethereum, tokenId: nil),
            assetFeeBalance: .mock(available: .zero),
        )

        await #expect(throws: AnyError.self) {
            try await provider.load(request: .mock(), metadata: metadata, selection: .preset(.normal))
        }
    }
}

private extension ConfirmTransferInputProvider {
    static func mock(transaction: Result<TransferTransactionData, Error>) -> ConfirmTransferInputProvider {
        ConfirmTransferInputProvider(
            transferTransactionProvider: TransferTransactionProviderMock(result: transaction),
            feeAssetProvider: FeeAssetProviderMock(),
        )
    }
}
