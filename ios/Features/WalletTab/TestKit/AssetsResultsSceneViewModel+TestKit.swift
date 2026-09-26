// Copyright (c). Gem Wallet. All rights reserved.

import protocol Gemstone.GemAssetSelectionServiceProtocol
import GemstoneServicesTestKit
import Primitives
import PrimitivesTestKit
import Store
import WalletTab

public extension AssetsResultsSceneViewModel {
    @MainActor
    static func mock(
        service: any GemAssetSelectionServiceProtocol = GemAssetSelectionServiceMock(),
        request: WalletSearchQuery = WalletSearchQuery(walletId: .mock(), searchBy: "usdc", types: [.asset]),
        onSelectAsset: @escaping (Asset) -> Void = { _ in },
    ) -> AssetsResultsSceneViewModel {
        AssetsResultsSceneViewModel(
            wallet: .mock(),
            service: service,
            request: request,
            title: "Results",
            onSelectAsset: onSelectAsset,
        )
    }
}
