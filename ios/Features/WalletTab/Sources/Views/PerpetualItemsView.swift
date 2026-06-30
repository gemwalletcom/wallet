// Copyright (c). Gem Wallet. All rights reserved.

import Perpetuals
import Primitives
import SwiftUI

struct PerpetualItemsView: View {
    let items: [PerpetualData]
    let onPin: (PerpetualData) -> Void
    let onSelect: (Asset) -> Void

    var body: some View {
        ForEach(items) { perpetualData in
            PerpetualListItem(
                perpetualData: perpetualData,
                onPin: onPin,
                onSelect: onSelect,
            )
        }
    }
}
