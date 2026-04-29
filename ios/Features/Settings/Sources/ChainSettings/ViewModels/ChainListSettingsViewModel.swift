// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import Primitives
import PrimitivesComponents

@Observable
@MainActor
public final class ChainListSettingsViewModel {
    public init() {}

    var emptyContent: EmptyContentTypeViewModel {
        EmptyContentTypeViewModel(type: .search(type: .networks))
    }
}

// MARK: - ChainFilterable

extension ChainListSettingsViewModel: ChainFilterable {
    public nonisolated var chains: [Chain] {
        AssetConfiguration.allChains.sortByRank()
    }
}
