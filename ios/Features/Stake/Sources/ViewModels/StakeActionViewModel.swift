// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.GemStakeAction
import Components
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
    let action: GemStakeAction
    let infoAction: InfoSheetAction?
    let isEnabled: Bool
}
