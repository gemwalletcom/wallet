// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GemstonePrimitives
import Localization
import Primitives
import PrimitivesComponents

public struct ChainListSettingsViewModel {
    public init() {}

    var title: String {
        Localized.Settings.Networks.title
    }

    var emptyContent: EmptyContentTypeViewModel {
        EmptyContentTypeViewModel(type: .search(type: .networks))
    }
}

// MARK: - ChainFilterable

extension ChainListSettingsViewModel: ChainFilterable {
    public var chains: [Chain] {
        AssetConfiguration.allChains.sortByRank()
    }
}
