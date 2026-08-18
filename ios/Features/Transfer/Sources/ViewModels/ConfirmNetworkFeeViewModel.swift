// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Primitives
import PrimitivesComponents

struct ConfirmNetworkFeeViewModel: ItemModelProvidable {
    private let state: StateViewType<ConfirmTransferInput>
    private let feeModel: NetworkFeeSceneViewModel
    private let infoAction: VoidAction

    init(
        state: StateViewType<ConfirmTransferInput>,
        feeModel: NetworkFeeSceneViewModel,
        infoAction: VoidAction,
    ) {
        self.state = state
        self.feeModel = feeModel
        self.infoAction = infoAction
    }
}

// MARK: - ItemModelProvidable

extension ConfirmNetworkFeeViewModel {
    var itemModel: ConfirmTransferItemModel {
        .networkFee(
            .init(
                title: feeModel.title,
                subtitle: networkFeeValue,
                placeholders: [.subtitle],
                infoAction: infoAction,
            ),
            selectable: feeModel.showFeeDetails && !state.isError,
        )
    }
}

// MARK: - Private

extension ConfirmNetworkFeeViewModel {
    private var networkFeeValue: String? {
        if state.isError { return "-" }
        return feeModel.fiatValue ?? feeModel.value
    }
}
