// Copyright (c). Gem Wallet. All rights reserved.

import Components
import enum Gemstone.GemListRow
import enum Gemstone.GemTransactionHeader
import PrimitivesComponents

public enum TransactionItemModel {
    case fee(ListItemModel)
    case header(GemTransactionHeader)
    case swapProgress(TransactionSwapProgressItemModel)
    case participant(AddressListItemViewModel)
    case row(GemListRow)
    case swapAgain(text: String)
    case empty
}
