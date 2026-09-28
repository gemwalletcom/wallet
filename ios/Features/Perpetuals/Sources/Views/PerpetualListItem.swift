// Copyright (c). Gem Wallet. All rights reserved.

import Components
import struct Gemstone.GemAssetItemRow
import Primitives
import PrimitivesComponents
import SwiftUI

struct PerpetualListItem: View {
    let perpetualData: PerpetualData
    let row: GemAssetItemRow
    let onPin: (PerpetualData) -> Void
    let onSelect: (Asset) -> Void

    var body: some View {
        NavigationCustomLink(
            with: ListAssetItemView(row: row),
            action: { onSelect(perpetualData.asset) },
        )
        .listRowInsets(.assetListRowInsets)
        .contextMenu(
            [
                .pin(
                    isPinned: perpetualData.metadata.isPinned,
                    onPin: {
                        onPin(perpetualData)
                    },
                ),
            ],
        )
    }
}
