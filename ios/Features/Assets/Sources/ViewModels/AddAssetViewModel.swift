// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import struct Gemstone.GemAssetInfoRow
import Localization
import Primitives
import PrimitivesComponents

struct AddAssetViewModel {
    let rows: [GemAssetInfoRow]
    let link: BlockExplorerLink?

    var explorerText: String? {
        link.map { Localized.Transaction.viewOn($0.name) }
    }

    var explorerUrl: URL? {
        link?.url
    }
}
