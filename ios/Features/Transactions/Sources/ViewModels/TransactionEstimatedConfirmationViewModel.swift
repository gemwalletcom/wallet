// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Formatters
import Localization
import Primitives

struct TransactionEstimatedConfirmationViewModel {
    private let seconds: UInt32?
    private let onInfoAction: VoidAction

    init(seconds: UInt32?, onInfoAction: VoidAction) {
        self.seconds = seconds
        self.onInfoAction = onInfoAction
    }
}

extension TransactionEstimatedConfirmationViewModel: ItemModelProvidable {
    var itemModel: TransactionItemModel {
        guard
            let seconds,
            seconds > 0
        else {
            return .empty
        }
        return .listItem(ListItemModel(
            title: Localized.Transaction.estimatedConfirmation,
            subtitle: EstimatedConfirmationFormatter().string(seconds: seconds),
            infoAction: onInfoAction,
        ))
    }
}
