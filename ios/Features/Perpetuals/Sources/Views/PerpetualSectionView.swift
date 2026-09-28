// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.GemAssetItemRow
import struct Gemstone.GemPerpetualMarketItem
import func Gemstone.perpetualMarketRows
import GemstonePrimitives
import Primitives
import SwiftUI

public struct PerpetualSectionView: View {
    private let items: [(data: PerpetualData, row: GemAssetItemRow)]
    private let onPin: (PerpetualData) -> Void
    private let onSelect: (Asset) -> Void

    public init(
        perpetuals: [PerpetualData],
        onPin: @escaping (PerpetualData) -> Void,
        onSelect: @escaping (Asset) -> Void,
    ) {
        items = Array(zip(perpetuals, perpetualMarketRows(markets: perpetuals.map { $0.toGem() })))
        self.onPin = onPin
        self.onSelect = onSelect
    }

    public init(
        items: [GemPerpetualMarketItem],
        onPin: @escaping (PerpetualData) -> Void,
        onSelect: @escaping (Asset) -> Void,
    ) {
        self.items = items.map { ($0.data.toPrimitives(), $0.row) }
        self.onPin = onPin
        self.onSelect = onSelect
    }

    public var body: some View {
        ForEach(items, id: \.data.id) { data, row in
            PerpetualListItem(
                perpetualData: data,
                row: row,
                onPin: onPin,
                onSelect: onSelect,
            )
        }
    }
}
