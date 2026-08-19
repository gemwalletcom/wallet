// Copyright (c). Gem Wallet. All rights reserved.

@testable import GemstonePrimitives
import Primitives
import PrimitivesTestKit
import Testing

struct TransferDataTypeTests {
    @Test
    func feeAsset() {
        #expect(TransferDataType.transfer(.tempoUSDC()).feeAsset == Asset.tempoPathUSD())
        #expect(TransferDataType.transfer(.mockEthereumUSDT()).feeAsset == Asset.mockEthereum())
        #expect(TransferDataType.transfer(.mockEthereum()).feeAsset == Asset.mockEthereum())
        #expect(TransferDataType.transfer(.hypercoreSpotUSDC()).feeAsset == Asset.hypercoreSpotUSDC())
        #expect(TransferDataType.perpetual(.hypercoreUSDC(), .mockOpen()).feeAsset == Asset.hypercoreUSDC())
    }
}
