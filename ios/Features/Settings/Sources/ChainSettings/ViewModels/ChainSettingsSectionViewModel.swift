// Copyright (c). Gem Wallet. All rights reserved.

struct ChainSettingsSectionViewModel: Identifiable {
    enum Kind {
        case nodes
        case explorer
    }

    let id: String
    let title: String
    let kind: Kind
}
