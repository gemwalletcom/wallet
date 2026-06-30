// Copyright (c). Gem Wallet. All rights reserved.

import Primitives
import SwiftUI

struct PerpetualSectionView: View {
    let perpetuals: [PerpetualData]
    let onPin: (PerpetualData) -> Void
    let onSelect: (Asset) -> Void

    var body: some View {
        ForEach(perpetuals) { perpetualData in
            PerpetualListItem(
                perpetualData: perpetualData,
                onPin: onPin,
                onSelect: onSelect,
            )
        }
    }
}
