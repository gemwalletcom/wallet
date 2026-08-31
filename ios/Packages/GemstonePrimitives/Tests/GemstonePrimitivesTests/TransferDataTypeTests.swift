// Copyright (c). Gem Wallet. All rights reserved.

@testable import GemstonePrimitives
import class Gemstone.GemTransferService
import Primitives
import PrimitivesTestKit
import Testing

struct TransferDataTypeTests {
    private let transferService = GemTransferService()

    @Test
    func feeAsset() {
        #expect(TransferDataType.transfer(.mockTempoUSDC()).feeAsset(transferService: transferService) == .mockTempoUSDC())
        #expect(TransferDataType.transfer(.mockEthereumUSDT()).feeAsset(transferService: transferService) == Asset.mockEthereum())
        #expect(TransferDataType.transfer(.mockEthereum()).feeAsset(transferService: transferService) == Asset.mockEthereum())
        #expect(TransferDataType.transfer(Asset.mockHypercoreSpotUSDC()).feeAsset(transferService: transferService) == Asset.mockHypercoreSpotUSDC())
        #expect(TransferDataType.perpetual(Asset.mockHypercoreUSDC(), .mockOpen()).feeAsset(transferService: transferService) == Asset.mockHypercoreUSDC())
    }
}
