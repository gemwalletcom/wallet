// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.GemStakeSection
import PrimitivesComponents

extension GemStakeSection: @retroactive Identifiable {
    public var id: Self { self }
}
