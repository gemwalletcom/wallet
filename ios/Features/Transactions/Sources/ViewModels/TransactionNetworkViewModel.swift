// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import Localization
import Primitives
import PrimitivesComponents

struct TransactionNetworkViewModel {
    private let chain: Chain

    init(chain: Chain) {
        self.chain = chain
    }
}

// MARK: - ItemModelProvidable

extension TransactionNetworkViewModel: ItemModelProvidable {
    var itemModel: TransactionItemModel {
        .network(
            title: Localized.Transfer.network,
            subtitle: chain.asset.name,
            image: AssetIdViewModel(assetId: chain.assetId).networkAssetImage,
        )
    }
}
