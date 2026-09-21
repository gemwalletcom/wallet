// Copyright (c). Gem Wallet. All rights reserved.

import Components
import enum Gemstone.GemStakeDestination
import enum Gemstone.GemStakeSection
import InfoSheet
import PrimitivesComponents

struct StakeSectionViewModel: Identifiable {
    let section: GemStakeSection
    let title: String

    var id: String { String(describing: section) }
}

struct StakeActionViewModel: Identifiable {
    let id: String
    let model: ListItemModel
    let destination: GemStakeDestination
    let infoAction: InfoSheetAction?
    let isEnabled: Bool
}
