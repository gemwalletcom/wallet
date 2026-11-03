// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GRDB
import Primitives

extension AssetRecord {
    static func textSearchFilter(query: String) -> SQLExpression {
        Columns.symbol.like("%%\(query)%%") ||
            Columns.name.like("%%\(query)%%") ||
            Columns.tokenId.like("%%\(query)%%") ||
            (
                Columns.type == AssetType.native.rawValue &&
                    Columns.chain.like("%%\(query)%%")
            )
    }

    static var fiatValue: SQLExpression {
        TableAlias(name: BalanceRecord.databaseTableName)[BalanceRecord.Columns.totalAmount] * (TableAlias(name: PriceRecord.databaseTableName)[PriceRecord.Columns.price] ?? 0)
    }
}
