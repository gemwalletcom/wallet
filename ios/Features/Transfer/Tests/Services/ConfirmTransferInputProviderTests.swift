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

        let input = try await provider.load(request: .mock(), metadata: .mock(), selection: .preset(.normal))

        #expect(input.feeRates.count == 1)
        #expect(input.transactionData.fee.fee == 1)
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
}

private extension ConfirmTransferInputProvider {
    static func mock(transaction: Result<TransferTransactionData, Error>) -> ConfirmTransferInputProvider {
        ConfirmTransferInputProvider(
            transferTransactionProvider: TransferTransactionProviderMock(result: transaction),
        )
    }
}
