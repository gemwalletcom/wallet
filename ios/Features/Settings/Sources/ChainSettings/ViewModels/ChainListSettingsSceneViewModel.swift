// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import struct Gemstone.GemChainRow
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

    func chainRows(for query: String) -> [GemChainRow] {
        GemChainService.shared.chainRows(chains: nil, query: query)
    }

    var serviceStatusListItem: ListItemModel {
        ListItemModel(
            title: Localized.Transaction.status,
            imageStyle: .asset(assetImage: AssetImage.image(Images.Logo.logo)),
        )
    }
}
