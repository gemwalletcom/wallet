// Copyright (c). Gem Wallet. All rights reserved.

import Components
import enum Gemstone.GemConfirmRowContent
import PrimitivesComponents

extension GemConfirmRowContent {
    var itemModel: ConfirmTransferItemModel {
        switch self {
        case let .row(row): .row(row)
        case let .recipient(row): .recipient(row)
        case let .paymentAsset(title, symbol, selectable, _): .paymentAsset(ListItemModel(title: title.text, subtitle: symbol), selectable: selectable)
        case .details: .empty
        }
    }

    func item(at index: Int) -> ConfirmTransferItem {
        switch self {
        case .row, .recipient, .paymentAsset: .row(index)
        case .details: .details
        }
    }
}
