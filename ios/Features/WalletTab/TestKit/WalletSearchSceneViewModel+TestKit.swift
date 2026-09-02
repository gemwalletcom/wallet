// Copyright (c). Gem Wallet. All rights reserved.

import protocol Gemstone.GemAssetSelectionServiceProtocol
import class Gemstone.GemRecentActivityService
import GemstoneServices
import GemstoneServicesTestKit
import Primitives
import PrimitivesTestKit
import Recents
import Store
import StoreTestKit
import WalletTab

public extension WalletSearchSceneViewModel {
    @MainActor
    static func mock(
        wallet: Wallet = .mock(),
        service: any GemAssetSelectionServiceProtocol = GemAssetSelectionServiceMock(),
    ) -> WalletSearchSceneViewModel {
        WalletSearchSceneViewModel(
            wallet: wallet,
            service: service,
            recentModel: RecentAssetsModel(
                walletId: wallet.id,
                types: RecentActivityType.allCases,
                service: GemRecentActivityService(store: GemstoneRecentActivityStore(store: .mock()), session: .mock()),
            ),
            onDismissSearch: {},
            onSelectAssetAction: { _ in },
            onAddToken: {},
        )
    }
}
