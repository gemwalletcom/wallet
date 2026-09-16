// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.GemTransactionFilter

extension GemTransactionFilter: @retroactive Identifiable {
    public var id: Self { self }
}
