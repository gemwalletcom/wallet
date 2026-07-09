// Copyright (c). Gem Wallet. All rights reserved.

import Primitives
@testable import PrimitivesComponents
import PrimitivesComponentsTestKit
import PrimitivesTestKit
import SwiftUI
import Testing

struct ListAssetItemViewModelTests {
    @Test
    func receiveCollectionUsesNetworkName() {
        let ton = Asset.mock(
            id: AssetId(chain: .ton, tokenId: nil),
            name: "Gram",
            symbol: "GRAM",
            decimals: 9,
            type: .native,
        )
        let assetDataModel = AssetDataViewModel.mock(assetData: .mock(asset: ton))

        let collectionModel = ListAssetItemViewModel(
            showBalancePrivacy: .constant(false),
            assetDataModel: assetDataModel,
            type: .copy(.collection),
        )
        let assetModel = ListAssetItemViewModel(
            showBalancePrivacy: .constant(false),
            assetDataModel: assetDataModel,
            type: .copy(.asset),
        )

        #expect(collectionModel.name == "TON")
        #expect(assetModel.name == "Gram")
    }
}
