// Copyright (c). Gem Wallet. All rights reserved.

@testable import GemstonePrimitives
import Primitives
import PrimitivesTestKit
import Testing

struct TransferDataTypeTests {
    @Test
    func feeAsset() {
        #expect(TransferDataType.transfer(.mockTempoUSDC()).feeAsset == .mockTempoUSDC())
        #expect(TransferDataType.transfer(.mockEthereumUSDT()).feeAsset == Asset.mockEthereum())
        #expect(TransferDataType.transfer(.mockEthereum()).feeAsset == Asset.mockEthereum())
        #expect(TransferDataType.transfer(Asset.mockHypercoreSpotUSDC()).feeAsset == Asset.mockHypercoreSpotUSDC())
        #expect(TransferDataType.perpetual(Asset.mockHypercoreUSDC(), .mockOpen()).feeAsset == Asset.mockHypercoreUSDC())
    }
}
