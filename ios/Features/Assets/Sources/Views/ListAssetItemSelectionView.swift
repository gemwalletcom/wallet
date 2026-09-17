// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Primitives
import PrimitivesComponents
import SwiftUI

struct ListAssetItemSelectionView: View {
    private let assetData: AssetData
    private let itemsModel: ListAssetItemsViewModel
    private let action: (ListAssetItemAction, AssetData) -> Void

    init(
        assetData: AssetData,
        itemsModel: ListAssetItemsViewModel,
        action: @escaping (ListAssetItemAction, AssetData) -> Void,
    ) {
        self.assetData = assetData
        self.itemsModel = itemsModel
        self.action = action
    }

    var body: some View {
        ListAssetItemView(model: itemsModel.item(assetData, action: { action($0, assetData) }))
    }
}
