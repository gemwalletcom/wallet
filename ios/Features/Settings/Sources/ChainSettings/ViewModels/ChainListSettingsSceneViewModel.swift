// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import class Gemstone.GemChainService
import protocol Gemstone.GemChainSettingsServiceProtocol
import GemstonePrimitives
import Localization
import Primitives
import PrimitivesComponents
import Style

@Observable
@MainActor
public final class ChainListSettingsSceneViewModel {
    private let service: any GemChainSettingsServiceProtocol

    public init(service: any GemChainSettingsServiceProtocol) {
        self.service = service
    }

    var emptyContent: EmptyStateViewModel {
        EmptyStateViewModel(kind: .searchNetworks)
    }

    func filterChains(for query: String) -> [Chain] {
        GemChainService.shared.getChains(query: query).map { Chain(core: $0) }
    }

    var serviceStatusListItem: ListItemModel {
        ListItemModel(
            title: Localized.Transaction.status,
            imageStyle: .asset(assetImage: AssetImage.image(Images.Logo.logo)),
        )
    }
}
