// Copyright (c). Gem Wallet. All rights reserved.

import Localization

struct ChainSettingsSectionViewModel: Identifiable {
    enum Kind: String, CaseIterable {
        case nodes
        case explorer
    }

    let kind: Kind

    var id: String { kind.rawValue }

    var title: String {
        switch kind {
        case .nodes: Localized.Settings.Networks.source
        case .explorer: Localized.Settings.Networks.explorer
        }
    }
}
