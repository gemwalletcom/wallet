// Copyright (c). Gem Wallet. All rights reserved.

import Components
import enum Gemstone.GemListRow
import PrimitivesComponents

public enum TransactionItemModel {
    case fee(ListItemModel)
    case header(TransactionHeaderType)
    case swapProgress(TransactionSwapProgressItemModel)
    case participant(AddressListItemViewModel)
    case row(GemListRow)
    case swapAgain(text: String)
    case empty
}
