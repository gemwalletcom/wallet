// Copyright (c). Gem Wallet. All rights reserved.

import Components
import struct Gemstone.GemAddressRow
import enum Gemstone.GemListRow
import enum Gemstone.GemTransactionHeader
import PrimitivesComponents

public enum TransactionItemModel {
    case fee(ListItemModel)
    case header(GemTransactionHeader)
    case swapProgress(TransactionSwapProgressItemModel)
    case participant(GemAddressRow)
    case row(GemListRow)
    case swapAgain(text: String)
    case empty
}
