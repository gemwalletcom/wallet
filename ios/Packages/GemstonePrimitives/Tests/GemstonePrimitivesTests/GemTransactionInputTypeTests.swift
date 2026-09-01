// Copyright (c). Gem Wallet. All rights reserved.

@testable import GemstonePrimitives
import enum Gemstone.GemTransactionInputType
import Primitives
import PrimitivesTestKit
import Testing

struct GemTransactionInputTypeTests {
    @Test
    func feeAsset() {
        #expect(GemTransactionInputType.transfer(.mockTempoUSDC()).feeAsset().map() == .mockTempoUSDC())
        #expect(GemTransactionInputType.transfer(.mockEthereumUSDT()).feeAsset().map() == Asset.mockEthereum())
        #expect(GemTransactionInputType.transfer(.mockEthereum()).feeAsset().map() == Asset.mockEthereum())
        #expect(GemTransactionInputType.transfer(Asset.mockHypercoreSpotUSDC()).feeAsset().map() == Asset.mockHypercoreSpotUSDC())
        #expect(GemTransactionInputType.perpetual(Asset.mockHypercoreUSDC(), .mockOpen()).feeAsset().map() == Asset.mockHypercoreUSDC())
    }
}
