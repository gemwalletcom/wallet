// Copyright (c). Gem Wallet. All rights reserved.

import Components
import struct Gemstone.GemTransactionStatus
import Foundation
import Localization
import Primitives
import PrimitivesComponents
import Style
import SwiftUI

struct TransactionStatusViewModel {
    private let status: GemTransactionStatus
    private let state: TransactionState
    private let onInfoAction: VoidAction

    init(
        status: GemTransactionStatus,
        state: TransactionState,
        onInfoAction: VoidAction,
    ) {
        self.status = status
        self.state = state
        self.onInfoAction = onInfoAction
    }

    private var stateViewModel: TransactionStateViewModel {
        TransactionStateViewModel(state: state, tone: status.tone)
    }
}

// MARK: - ItemModelProvidable

extension TransactionStatusViewModel: ItemModelProvidable {
    var itemModel: TransactionItemModel {
        .listItem(ListItemModel(
            title: Localized.Transaction.status,
            subtitle: stateViewModel.title,
            subtitleStyle: TextStyle(font: .callout, color: stateViewModel.color),
            subtitleTagType: status.showsProgress ? .progressView() : .none,
            infoAction: onInfoAction,
        ))
    }
}
