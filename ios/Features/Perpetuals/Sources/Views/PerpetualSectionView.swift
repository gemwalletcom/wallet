// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.GemPerpetualMarketItem
import GemstonePrimitives
import Primitives
import SwiftUI

public struct PerpetualSectionView: View {
    private let items: [(data: PerpetualData, model: PerpetualItemViewModel)]
    private let onPin: (PerpetualData) -> Void
    private let onSelect: (Asset) -> Void

    public init(
        perpetuals: [PerpetualData],
        onPin: @escaping (PerpetualData) -> Void,
        onSelect: @escaping (Asset) -> Void,
    ) {
        items = PerpetualItemViewModel.items(perpetuals)
        self.onPin = onPin
        self.onSelect = onSelect
    }

    public init(
        items: [GemPerpetualMarketItem],
        onPin: @escaping (PerpetualData) -> Void,
        onSelect: @escaping (Asset) -> Void,
    ) {
        self.items = items.map { ($0.data.toPrimitives(), PerpetualItemViewModel(row: $0.row)) }
        self.onPin = onPin
        self.onSelect = onSelect
    }

    public var body: some View {
        ForEach(items, id: \.data.id) { data, model in
            PerpetualListItem(
                perpetualData: data,
                model: model,
                onPin: onPin,
                onSelect: onSelect,
            )
        }
    }
}
