// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.GemAcquireOption

extension GemAcquireOption: @retroactive Identifiable {
    public var id: Self { self }
}
