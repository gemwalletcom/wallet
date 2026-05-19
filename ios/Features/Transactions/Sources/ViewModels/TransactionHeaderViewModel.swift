// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Primitives
import PrimitivesComponents

struct TransactionHeaderViewModel {
    private let transaction: TransactionExtended
    private let infoModel: TransactionInfoViewModel

    init(
        transaction: TransactionExtended,
        infoModel: TransactionInfoViewModel,
    ) {
        self.transaction = transaction
        self.infoModel = infoModel
    }

    var headerType: TransactionHeaderType {
        TransactionHeaderTypeBuilder.build(
            infoModel: infoModel,
            transaction: transaction.transaction,
            metadata: TransactionExtendedMetadata(
                assets: transaction.assets,
                assetPrices: transaction.prices,
                metadata: transaction.transaction.metadata,
            ),
        )
    }

    var showClearHeader: Bool {
        switch headerType {
        case .amount, .nft, .asset, .assetValue: true
        case .swap: false
        }
    }
}

// MARK: - ItemModelProvidable

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
