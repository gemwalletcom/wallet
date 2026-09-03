// Copyright (c). Gem Wallet. All rights reserved.

import GemstonePrimitives
import Primitives
import Testing

struct SelectAssetTypeTests {
    @Test
    func action() {
        #expect(SelectAssetType.swap(.pay).action == .swapPay)
        #expect(SelectAssetType.swap(.receive(chains: [], assetIds: [])).action == .swapReceive)
        #expect(SelectAssetType.receive(.asset).action == .receive)
        #expect(SelectAssetType.buy.action == .buy)
        #expect(SelectAssetType.send(.none).action == .send)
        #expect(SelectAssetType.manage.action == nil)
    }
}
