// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import protocol Gemstone.GemChainServiceProtocol
import GemstonePrimitives
import Primitives
import PrimitivesComponents

@Observable
@MainActor
public final class ChainListSettingsViewModel {
    private let chainService: any GemChainServiceProtocol

    public init(chainService: any GemChainServiceProtocol) {
        self.chainService = chainService
    }

    var emptyContent: EmptyContentTypeViewModel {
        EmptyContentTypeViewModel(type: .search(type: .networks))
    }

    func filterChains(for query: String) -> [Chain] {
        chainService.getChains(query: query).map { Chain(core: $0) }
    }
}
