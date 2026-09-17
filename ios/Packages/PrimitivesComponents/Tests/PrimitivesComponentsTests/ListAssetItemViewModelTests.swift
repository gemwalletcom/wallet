// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.GemSelectAssetType
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
            rowStyle: GemSelectAssetType.receiveCollection.flow().rowStyle,
        )
        let assetModel = ListAssetItemViewModel(
            showBalancePrivacy: .constant(false),
            assetDataModel: assetDataModel,
            rowStyle: GemSelectAssetType.receive.flow().rowStyle,
        )

        #expect(collectionModel.name == "TON")
        #expect(assetModel.name == "Gram")
    }
}
