// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.GemStakeDelegationItem
import enum Gemstone.GemStakeSection
import PrimitivesComponents

extension GemStakeSection: @retroactive Identifiable {
    public var id: Self { self }
}

extension GemStakeDelegationItem: @retroactive Identifiable {}
