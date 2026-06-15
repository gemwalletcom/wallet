// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Primitives

struct ConfirmNetworkFeeViewModel: ItemModelProvidable {
    private let state: StateViewType<TransactionInputViewModel>
    private let title: String
    private let value: String?
    private let fiatValue: String?
    private let selectable: Bool
    private let infoAction: VoidAction

    init(
        state: StateViewType<TransactionInputViewModel>,
        title: String,
        value: String?,
        fiatValue: String?,
        selectable: Bool,
        infoAction: VoidAction,
    ) {
        self.state = state
        self.title = title
        self.value = value
        self.fiatValue = fiatValue
        self.selectable = selectable
        self.infoAction = infoAction
    }
}

// MARK: - ItemModelProvidable

extension ConfirmNetworkFeeViewModel {
    var itemModel: ConfirmTransferItemModel {
        .networkFee(
            .init(
                title: title,
                subtitle: networkFeeValue,
                placeholders: [.subtitle],
                infoAction: infoAction,
            ),
            selectable: selectable && !state.isError,
        )
    }
}

// MARK: - Private

extension ConfirmNetworkFeeViewModel {
    private var networkFeeValue: String? {
        if state.isError { return "-" }
        return fiatValue ?? value
    }
}
