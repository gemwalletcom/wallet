// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import struct Gemstone.GemConfirmData
import struct Gemstone.GemFeeRate
import GemstonePrimitivesTestKit
@testable import Primitives
import PrimitivesTestKit
import class Gemstone.GemTransferService
import Testing
@testable import Transfer
import TransferTestKit
import Validators
import class Gemstone.GemFeeService

struct ConfirmTransferInputProviderTests {
    @Test
    func loadReturnsInputWithRatesAndFee() async throws {
        let provider = ConfirmTransferInputProvider.mock(transaction: .success(
            .mock(input: TransferData.mock().confirmInput(from: .mock()), feeRates: [GemFeeRate(priority: .normal, gasPriceType: .regular(gasPrice: "1"))]),
        ))

        let result = try await provider.load()

        #expect(result.feeRates.count == 1)
        #expect(result.input.fee.fee == 1)
    }

    @Test
    func loadUsesFeeAssetFromTransactionFee() async throws {
        let feeAsset = Asset.mockHypercoreUSDC()
        let feeAssetBalance = Balance.mock(available: 42)
        let feeAssetPrice = Price.mock(price: 1)
        let provider = ConfirmTransferInputProvider(
            transferTransactionProvider: TransferTransactionProviderMock(result: .success(
                .mock(input: TransferData.mock().confirmInput(from: .mock()), fee: .mock(feeAsset: feeAsset.id.identifier)),
            )),
            feeAssetProvider: FeeAssetProviderMock(asset: feeAsset, balance: feeAssetBalance, price: feeAssetPrice),
            feeService: GemFeeService(),
            transferService: GemTransferService(),
        )

        let result = try await provider.load()

        #expect(result.input.feeAsset == feeAsset)
        #expect(result.metadata.assetFeeBalance == feeAssetBalance)
        #expect(result.metadata.feeAssetId == feeAsset.id)
        #expect(result.metadata.feePrice == feeAssetPrice)
    }

    @Test
    func loadUsesSelectedFeeAsset() async throws {
        let selectedAsset = Asset.mockTempoUSDC()
        let transferProvider = TransferTransactionProviderMock(result: .success(
            .mock(input: TransferData.mock().confirmInput(from: .mock()), fee: .mock(feeAsset: selectedAsset.id.identifier)),
        ))
        let provider = ConfirmTransferInputProvider(
            transferTransactionProvider: transferProvider,
            feeAssetProvider: FeeAssetProviderMock(
                asset: selectedAsset,
                balance: .mock(available: 42),
                price: .mock(price: 1),
            ),
            feeService: GemFeeService(),
            transferService: GemTransferService(),
        )

        let result = try await provider.load(
            request: .mock(data: .mock(type: .transfer(.mockTempoPathUSD()))),
            metadata: .mock(),
            selection: .preset(.normal),
            feeAssetSelection: .selected(selectedAsset.id),
        )

        #expect(transferProvider.loadedFeeAssetId == selectedAsset.id)
        #expect(result.input.feeAsset == selectedAsset)
        #expect(result.metadata.feeAssetId == selectedAsset.id)
    }

    @Test
    func loadMapsPreloadFailureToInsufficientNetworkFee() async {
        let provider = ConfirmTransferInputProvider.mock(transaction: .failure(AnyError("network")))
        let metadata = TransferDataMetadata.mock(
            feeAssetId: AssetId(chain: .ethereum, tokenId: nil),
            assetFeeBalance: .mock(available: .zero),
        )

        await #expect(throws: TransferAmountCalculatorError.self) {
            try await provider.load(metadata: metadata)
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
            try await provider.load(metadata: metadata)
        }
    }

    @Test
    func loadRethrowsPreloadFailureForTempoFeeAsset() async {
        let provider = ConfirmTransferInputProvider.mock(transaction: .failure(AnyError("network")))
        let metadata = TransferDataMetadata.mock(
            feeAssetId: Asset.mockTempoPathUSD().id,
            assetFeeBalance: .mock(available: .zero),
        )

        await #expect(throws: AnyError.self) {
            try await provider.load(metadata: metadata)
        }
    }

    @Test
    func loadRethrowsFeeAssetFailure() async {
        let provider = ConfirmTransferInputProvider(
            transferTransactionProvider: TransferTransactionProviderMock(result: .success(
                .mock(input: TransferData.mock().confirmInput(from: .mock())),
            )),
            feeAssetProvider: FeeAssetProviderMock(error: "fee asset"),
            feeService: GemFeeService(),
            transferService: GemTransferService(),
        )
        let metadata = TransferDataMetadata.mock(
            feeAssetId: AssetId(chain: .ethereum, tokenId: nil),
            assetFeeBalance: .mock(available: .zero),
        )

        await #expect(throws: AnyError.self) {
            try await provider.load(metadata: metadata)
        }
    }
}

private extension ConfirmTransferInputProvider {
    func load(metadata: TransferDataMetadata = .mock()) async throws -> ConfirmTransferPreload {
        try await load(
            request: .mock(),
            metadata: metadata,
            selection: .preset(.normal),
            feeAssetSelection: .automatic,
        )
    }

    static func mock(transaction: Result<GemConfirmData, Error>) -> ConfirmTransferInputProvider {
        ConfirmTransferInputProvider(
            transferTransactionProvider: TransferTransactionProviderMock(result: transaction),
            feeAssetProvider: FeeAssetProviderMock(),
            feeService: GemFeeService(),
            transferService: GemTransferService(),
        )
    }
}
