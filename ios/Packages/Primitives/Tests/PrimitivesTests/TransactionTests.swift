// Copyright (c). Gem Wallet. All rights reserved.

import Primitives
import PrimitivesTestKit
import Testing

struct TransactionTests {
    @Test
    func associatedAssetIdsIncludePrimaryAndFeeAssetsWithoutMetadata() {
        let primaryAsset = AssetId.mock(.ethereum)
        let transaction = Transaction.mock(
            type: .swap,
            assetId: primaryAsset,
            metadata: nil,
        )

        #expect(transaction.associatedAssetIds == [primaryAsset, .mock(.bitcoin)])
    }
}
