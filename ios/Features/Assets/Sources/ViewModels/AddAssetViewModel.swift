// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import struct Gemstone.GemAssetInfoRow
import Localization
import Primitives
import PrimitivesComponents

struct AddAssetViewModel {
    let rows: [GemAssetInfoRow]
    let link: BlockExplorerLink?

    func listItem(for row: GemAssetInfoRow) -> ListItemModel {
        ListItemModel(title: row.kind.title, subtitle: row.value)
    }

    var explorerListItem: ListItemModel? {
        explorerText.map { ListItemModel(title: $0) }
    }

    var explorerText: String? {
        link.map { Localized.Transaction.viewOn($0.name) }
    }

    var explorerUrl: URL? {
        link?.url
    }
}
