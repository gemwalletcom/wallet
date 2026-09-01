// Copyright (c). Gem Wallet. All rights reserved.

@testable import GemstonePrimitives
import enum Gemstone.GemTransactionInputType
import class Gemstone.GemTransferService
import Primitives
import PrimitivesTestKit
import Testing

struct GemTransactionInputTypeTests {
    private let transferService = GemTransferService()

    @Test
    func feeAsset() {
        #expect(GemTransactionInputType.transfer(.mockTempoUSDC()).feeAsset(transferService: transferService) == .mockTempoUSDC())
        #expect(GemTransactionInputType.transfer(.mockEthereumUSDT()).feeAsset(transferService: transferService) == Asset.mockEthereum())
        #expect(GemTransactionInputType.transfer(.mockEthereum()).feeAsset(transferService: transferService) == Asset.mockEthereum())
        #expect(GemTransactionInputType.transfer(Asset.mockHypercoreSpotUSDC()).feeAsset(transferService: transferService) == Asset.mockHypercoreSpotUSDC())
        #expect(GemTransactionInputType.perpetual(Asset.mockHypercoreUSDC(), .mockOpen()).feeAsset(transferService: transferService) == Asset.mockHypercoreUSDC())
    }
}
