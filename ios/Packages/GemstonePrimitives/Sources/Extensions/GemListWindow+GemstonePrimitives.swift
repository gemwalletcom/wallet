// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.GemListWindow

public extension GemListWindow {
    static let transactions = GemListWindow(limit: GemConstants.listPageSize, maximum: GemConstants.transactionsListLimit)
}
