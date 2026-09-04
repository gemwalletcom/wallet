// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import Localization
import Primitives
import PrimitivesComponents
import Style

struct TransactionNetworkFeeViewModel {
    private let feeDisplay: AmountDisplay?
    private let onInfoAction: VoidAction

    init(
        feeDisplay: AmountDisplay?,
        onInfoAction: VoidAction = nil,
    ) {
        self.feeDisplay = feeDisplay
        self.onInfoAction = onInfoAction
    }
}

// MARK: - ItemModelProvidable

extension TransactionNetworkFeeViewModel: ItemModelProvidable {
    var itemModel: TransactionItemModel {
        .fee(
            ListItemModel(
                title: Localized.Transfer.networkFee,
                subtitle: feeDisplay?.fiat?.text ?? feeDisplay?.amount.text ?? Placeholder.empty,
                subtitleExtra: nil,
                infoAction: onInfoAction,
            ),
        )
    }
}
