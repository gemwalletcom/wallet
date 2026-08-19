// Copyright (c). Gem Wallet. All rights reserved.

@testable import GemstonePrimitives
import Primitives
import PrimitivesTestKit
import Testing

struct TransferDataTypeTests {
    @Test
    func feeAsset() {
        #expect(TransferDataType.transfer(.mockTempoUSDC()).feeAsset == .mockTempoUSDC())
    }
}
