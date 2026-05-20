// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Components
import Formatters
import Foundation
import Localization
import Primitives

struct TransactionRateViewModel {
    private let transaction: TransactionExtended
    private let direction: AssetRateFormatter.Direction

    init(
        transaction: TransactionExtended,
        direction: AssetRateFormatter.Direction,
    ) {
        self.transaction = transaction
        self.direction = direction
    }
}

// MARK: - ItemModelProvidable

extension TransactionRateViewModel: ItemModelProvidable {
    var itemModel: TransactionItemModel {
        guard
            let metadata = transaction.transaction.metadata?.decode(TransactionSwapMetadata.self),
            let fromAsset = transaction.assets.first(where: { $0.id == metadata.fromAsset }),
            let toAsset = transaction.assets.first(where: { $0.id == metadata.toAsset }),
            let fromValue = BigInt(metadata.fromValue),
            let toValue = BigInt(metadata.toValue),
            let rate = try? AssetRateFormatter().rate(
                fromAsset: fromAsset,
                toAsset: toAsset,
                fromValue: fromValue,
                toValue: toValue,
                direction: direction,
            )
        else {
            return .empty
        }

        return .rate(title: Localized.Buy.rate, value: rate)
    }
}
