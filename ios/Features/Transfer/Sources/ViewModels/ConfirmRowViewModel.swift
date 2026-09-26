// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import enum Gemstone.GemConfirmRowContent
import GemstonePrimitives
import Localization
import Primitives
import PrimitivesComponents

struct ConfirmRowViewModel {
    private let content: GemConfirmRowContent

    init(content: GemConfirmRowContent) {
        self.content = content
    }
}

// MARK: - Item Model

extension ConfirmRowViewModel {
    var itemModel: ConfirmTransferItemModel {
        switch content {
        case let .row(row):
            .row(row)
        case let .recipient(row):
            .recipient(row)
        case let .paymentAsset(symbol, selectable, _):
            .paymentAsset(ListItemModel(title: Localized.Transfer.payWith, subtitle: symbol), selectable: selectable)
        case .details:
            .empty
        }
    }
}

extension GemConfirmRowContent {
    func item(at index: Int) -> ConfirmTransferItem {
        switch self {
        case .row, .recipient, .paymentAsset: .row(index)
        case .details: .details
        }
    }
}
