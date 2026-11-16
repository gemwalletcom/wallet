// Copyright (c). Gem Wallet. All rights reserved.

import Components
import enum Gemstone.GemTransactionFilter
import PrimitivesComponents

extension GemTransactionFilter: @retroactive Identifiable {
    public var id: Self { self }
}

extension GemTransactionFilter {
    var listItem: ListItemModel {
        ListItemModel(title: title)
    }
}
