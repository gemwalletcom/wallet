// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import enum Gemstone.GemTransactionHeader
import Primitives
import PrimitivesComponents

struct TransactionHeaderViewModel {
    private let header: GemTransactionHeader
    private let currency: Currency

    init(header: GemTransactionHeader, currency: Currency) {
        self.header = header
        self.currency = currency
    }

    var headerType: TransactionHeaderType {
        header.headerType(currency: currency)
    }

    var showClearHeader: Bool {
        switch headerType {
        case .amount, .nft, .asset, .assetValue: true
        case .swap: false
        }
    }
}

extension TransactionHeaderViewModel: ItemModelProvidable {
    var itemModel: TransactionItemModel {
        .header(
            TransactionHeaderItemModel(
                headerType: headerType,
                showClearHeader: showClearHeader,
            ),
        )
    }
}
