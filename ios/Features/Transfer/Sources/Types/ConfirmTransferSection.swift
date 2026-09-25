// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import enum Gemstone.GemListRow
import Primitives
import PrimitivesComponents
import Swap
import SwiftUI

enum ConfirmTransferSectionType: String, Identifiable, Equatable {
    case header
    case warnings
    case details
    case balanceChanges
    case payload
    case fee
    case error

    var id: String {
        rawValue
    }
}

public enum ConfirmTransferItem: Identifiable, Hashable, Sendable {
    case header
    case warnings
    case row(Int)
    case verification
    case details
    case balanceChange(Int)
    case payload
    case networkFee
    case error

    public var id: Self {
        self
    }
}

public enum ConfirmTransferItemModel {
    case header(TransactionHeaderType, isReserved: Bool)
    case row(GemListRow)
    case recipient(AddressListItemViewModel)
    case paymentAsset(ListItemModel, selectable: Bool)
    case verification(ListItemModel)
    case swapDetails(SwapDetailsViewModel)
    case networkFee(ListItemModel, selectable: Bool)
    case perpetualDetails(PerpetualDetailsViewModel)
    case perpetualModifyPosition(GemListRow?)
    case warnings([GemListRow])
    case payload([SimulationPayloadFieldViewModel])
    case balanceChange(ConfirmBalanceChangeViewModel)
    case error(title: String, error: Error, onInfoAction: VoidAction)
    case empty
}

extension ConfirmTransferItemModel: ItemModelProvidable {
    public var itemModel: ConfirmTransferItemModel {
        self
    }
}

extension ListSection where T == ConfirmTransferItem {
    init(type: ConfirmTransferSectionType, _ items: [ConfirmTransferItem]) {
        self.init(type: type, values: items)
    }
}
