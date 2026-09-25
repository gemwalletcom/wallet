// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Primitives
import PrimitivesComponents
import SwiftUI

struct AssetItemsView: View {
    let items: [AssetData]
    let itemsModel: ListAssetItemsViewModel
    let contextMenuItems: (AssetData) -> [ContextMenuItemType]
    let onSelect: (Asset) -> Void

    var body: some View {
        ForEach(Array(zip(items, itemsModel.rows(items))), id: \.0.id) { assetData, row in
            NavigationCustomLink(
                with: ListAssetItemView(row: row)
                    .contextMenu(contextMenuItems(assetData)),
                action: { onSelect(assetData.asset) },
            )
        }
    }
}
