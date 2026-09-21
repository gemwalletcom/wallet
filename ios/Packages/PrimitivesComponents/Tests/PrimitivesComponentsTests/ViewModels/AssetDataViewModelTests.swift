// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Primitives
@testable import PrimitivesComponents
import PrimitivesComponentsTestKit
import PrimitivesTestKit
import Testing

struct AssetDataViewModelTests {
    @Test
    func balanceTextWithSymbol() {
        let model = AssetDataViewModel.mock(assetData: .mock(asset: .mockEthereum()))

        #expect(model.balanceTextWithSymbol(BigInt(1_000_000_000_000_000_000)) == "1 ETH")
        #expect(model.balanceTextWithSymbol(.zero) == "0 ETH")
    }
}
