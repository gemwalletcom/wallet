// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.GemStakeDelegationItem
import enum Gemstone.GemStakeDestination
import enum Gemstone.GemStakeSection
import GemstonePrimitives
import Primitives
import PrimitivesComponents

extension GemStakeSection: @retroactive Identifiable {
    public var id: Self { self }
}

extension GemStakeDelegationItem: @retroactive Identifiable {
    public var id: String {
        delegation.toPrimitives().id
    }
}
