// Copyright (c). Gem Wallet. All rights reserved.

import Components
import enum Gemstone.GemConfirmFeeRow
import Primitives
import PrimitivesComponents

struct ConfirmNetworkFeeViewModel: ItemModelProvidable {
    private let feeRow: GemConfirmFeeRow
    private let feeModel: NetworkFeeSceneViewModel
    private let infoAction: VoidAction

    init(
        feeRow: GemConfirmFeeRow,
        feeModel: NetworkFeeSceneViewModel,
        infoAction: VoidAction,
    ) {
        self.feeRow = feeRow
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
                subtitleExtra: feeRow == .unavailable ? nil : feeModel.feeAssetSymbol,
                placeholders: [.subtitle],
                infoAction: infoAction,
            ),
            selectable: feeModel.showFeeDetails && feeRow != .unavailable,
        )
    }
}

// MARK: - Private

extension ConfirmNetworkFeeViewModel {
    private var networkFeeValue: String? {
        switch feeRow {
        case .unavailable: "-"
        case .loading: nil
        case .ready: feeModel.fiatValue ?? feeModel.value
        }
    }
}
