// Copyright (c). Gem Wallet. All rights reserved.

import Components
import enum Gemstone.GemConfirmRowContent
import PrimitivesComponents

extension GemConfirmRowContent {
    var itemModel: ConfirmTransferItemModel {
        switch self {
        case let .row(row): .row(row)
        case let .recipient(row): .recipient(row)
        case let .paymentAsset(title, symbol, link, _): .paymentAsset(ListItemModel(title: title.text, subtitle: symbol), link: link)
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
