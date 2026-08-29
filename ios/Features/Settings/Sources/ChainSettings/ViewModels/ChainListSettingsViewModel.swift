// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import protocol Gemstone.GemChainServiceProtocol
import Primitives
import PrimitivesComponents

@Observable
@MainActor
public final class ChainListSettingsViewModel {
    public let chainService: any GemChainServiceProtocol

    public init(chainService: any GemChainServiceProtocol) {
        self.chainService = chainService
    }

    var emptyContent: EmptyContentTypeViewModel {
        EmptyContentTypeViewModel(type: .search(type: .networks))
    }
}

// MARK: - ChainFilterable

extension ChainListSettingsViewModel: ChainFilterable {}
