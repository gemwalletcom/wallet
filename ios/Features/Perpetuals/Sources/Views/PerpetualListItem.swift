// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Formatters
import GemstonePrimitives
import Primitives
import PrimitivesComponents
import SwiftUI

public struct PerpetualListItem: View {
    let perpetualData: PerpetualData
    let onPin: (PerpetualData) -> Void
    let onSelect: (Asset) -> Void

    public init(
        perpetualData: PerpetualData,
        onPin: @escaping (PerpetualData) -> Void,
        onSelect: @escaping (Asset) -> Void,
    ) {
        self.perpetualData = perpetualData
        self.onPin = onPin
        self.onSelect = onSelect
    }

    public var body: some View {
        NavigationCustomLink(
            with: ListAssetItemView(
                model: PerpetualItemViewModel(
                    model: PerpetualViewModel(perpetual: perpetualData.perpetual),
                ),
            ),
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
