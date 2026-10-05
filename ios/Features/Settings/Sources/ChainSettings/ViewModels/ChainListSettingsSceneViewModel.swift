// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import struct Gemstone.GemChainList
import class Gemstone.GemChainService
import GemstonePrimitives
import Localization
import Primitives
import PrimitivesComponents
import Style

@Observable
@MainActor
public final class ChainListSettingsSceneViewModel {
    public init() {}

    func chainList(for query: String) -> GemChainList {
        GemChainService.shared.chainList(chains: nil, query: query)
    }

    var serviceStatusListItem: ListItemModel {
        ListItemModel(
            title: Localized.Transaction.status,
            imageStyle: .asset(assetImage: AssetImage.image(Images.Logo.logo)),
        )
    }
}
