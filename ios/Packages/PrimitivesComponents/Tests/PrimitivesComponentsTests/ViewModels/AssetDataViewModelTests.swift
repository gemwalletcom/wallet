// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Primitives
@testable import PrimitivesComponents
import PrimitivesComponentsTestKit
import PrimitivesTestKit
import Testing

struct AssetDataViewModelTests {
    @Test
    func apr() {
        let model = AssetDataViewModel.mock(assetData: .mock(metadata: .mock(stakingApr: 5.0, earnApr: 3.0)))

        #expect(model.apr(for: .stake) == 5.0)
        #expect(model.apr(for: .earn) == 3.0)
        #expect(AssetDataViewModel.mock().apr(for: .stake) == nil)
        #expect(AssetDataViewModel.mock().apr(for: .earn) == nil)
    }

    @Test
    func balanceTextWithSymbol() {
        let model = AssetDataViewModel.mock(assetData: .mock(asset: .mockEthereum()))

        #expect(model.balanceTextWithSymbol(BigInt(1_000_000_000_000_000_000)) == "1 ETH")
        #expect(model.balanceTextWithSymbol(.zero) == "0 ETH")
    }
}
