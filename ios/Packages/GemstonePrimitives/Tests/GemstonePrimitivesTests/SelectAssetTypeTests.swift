// Copyright (c). Gem Wallet. All rights reserved.

import GemstonePrimitives
import Primitives
import Testing

struct SelectAssetTypeTests {
    @Test
    func swapReceiveCarriesThePayAsset() {
        let payAssetId = AssetId(chain: .ethereum)

        #expect(SelectAssetType.swap(.receive(payAssetId: payAssetId)).flowType == .swapReceive(payAssetId: payAssetId.identifier))
        #expect(SelectAssetType.swap(.receive(payAssetId: nil)).flowType == .swapReceive(payAssetId: nil))
        #expect(SelectAssetType.swap(.pay).flowType == .swapPay)
    }
}
