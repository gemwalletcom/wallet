// Copyright (c). Gem Wallet. All rights reserved.

import Components
import struct Gemstone.GemStakeDelegationItem
import enum Gemstone.GemStakeDestination
import enum Gemstone.GemStakeSection
import GemstonePrimitives
import InfoSheet
import Primitives
import PrimitivesComponents

struct StakeSectionViewModel: Identifiable {
    let section: GemStakeSection
    let title: String

    var id: String { String(describing: section) }
}

extension GemStakeDelegationItem: @retroactive Identifiable {
    public var id: String {
        Delegation(core: delegation).id
    }
}
