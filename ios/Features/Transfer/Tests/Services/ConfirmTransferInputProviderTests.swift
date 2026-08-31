// Copyright (c). Gem Wallet. All rights reserved.

import class Gemstone.GemAmountService
import class Gemstone.GemFeeService
import class Gemstone.GemTransferService
import enum Gemstone.GemConfirmError
import enum Gemstone.GemTransferAmountError
import enum Gemstone.GemTransferAmountResult
import struct Gemstone.GemConfirmPreload
import struct Gemstone.GemTransferAmount
import Foundation
import GemstonePrimitivesTestKit
import GemstoneServicesTestKit
import Primitives
import PrimitivesTestKit
import Testing
@testable import Transfer

struct ConfirmTransferInputProviderTests {
    @Test
    func mapsCorePreloadIntoTheConfirmInput() async throws {
        let feeAsset = Asset.mockEthereum()
        let preload = try await provider(
            .success(.mock(confirmData: .mock(fee: .mock(fee: "7", feeAsset: feeAsset.id.identifier)), feeAsset: feeAsset)),
        ).load(request: request(), metadata: .mock(), selection: .preset(.normal), feeAssetSelection: .automatic)

        #expect(preload.input.feeAsset == feeAsset)
        #expect(preload.input.fee.fee == 7)
        #expect(preload.metadata.feeAssetId == feeAsset.id)
        #expect(try preload.input.transferAmount.get().value == 1)
    }

    @Test
    func mapsAnAmountErrorIntoAFailedValidation() async throws {
        let feeAsset = Asset.mockEthereum()
        let error = GemTransferAmountError.InsufficientBalance(assetId: feeAsset.id.identifier, required: "10", available: "1")
        let preload = try await provider(
            .success(.mock(feeAsset: feeAsset, amount: .error(error: error))),
        ).load(request: request(), metadata: .mock(), selection: .preset(.normal), feeAssetSelection: .automatic)

        #expect(throws: (any Error).self) { try preload.input.transferAmount.get() }
    }

    @Test
    func rethrowsAScanFailureAsATransactionError() async {
        let subject = provider(.failure(GemConfirmError.ScanMalicious))

        await #expect(throws: ScanTransactionError.malicious) {
            try await subject.load(request: request(), metadata: .mock(), selection: .preset(.normal), feeAssetSelection: .automatic)
        }
    }

    private func request() -> ConfirmTransferRequest {
        let data = TransferData.mock()
        return .mock(wallet: .mock(accounts: [.mock(chain: data.chain)]), data: data)
    }

    private func provider(_ preload: Result<GemConfirmPreload, any Error>) -> ConfirmTransferInputProvider {
        ConfirmTransferInputProvider(
            confirmService: GemConfirmServiceMock(preload: preload),
            feeService: GemFeeService(),
            transferService: GemTransferService(),
            amountService: GemAmountService(),
        )
    }
}
