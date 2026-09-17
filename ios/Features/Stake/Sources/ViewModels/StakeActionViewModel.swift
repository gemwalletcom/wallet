// Copyright (c). Gem Wallet. All rights reserved.

import Components
import InfoSheet
import PrimitivesComponents

enum StakeSection: String, Identifiable {
    case manage
    case resources
    case delegations

    var id: String { rawValue }
}

struct StakeSectionViewModel: Identifiable {
    let section: StakeSection
    let title: String

    var id: String { section.id }
}

struct StakeActionViewModel: Identifiable {
    let id: String
    let model: ListItemModel
    let destination: any Hashable
    let infoAction: InfoSheetAction?
    let isEnabled: Bool
}
