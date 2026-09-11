// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Formatters
import Localization
import Primitives

struct TransactionEstimatedConfirmationViewModel {
    private let minutes: UInt32?
    private let onInfoAction: VoidAction

    init(minutes: UInt32?, onInfoAction: VoidAction) {
        self.minutes = minutes
        self.onInfoAction = onInfoAction
    }
}

extension TransactionEstimatedConfirmationViewModel: ItemModelProvidable {
    var itemModel: TransactionItemModel {
        guard let minutes else {
            return .empty
        }
        return .listItem(ListItemModel(
            title: Localized.Transaction.estimatedConfirmation,
            subtitle: EstimatedConfirmationFormatter().string(minutes: minutes),
            infoAction: onInfoAction,
        ))
    }
}
