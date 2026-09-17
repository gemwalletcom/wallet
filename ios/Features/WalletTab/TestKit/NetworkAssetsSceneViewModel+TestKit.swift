// Copyright (c). Gem Wallet. All rights reserved.

import protocol Gemstone.GemWalletHomeServiceProtocol
import GemstonePrimitivesTestKit
import Primitives
import PrimitivesTestKit
import WalletTab

public extension NetworkAssetsSceneViewModel {
    @MainActor
    static func mock(
        service: any GemWalletHomeServiceProtocol = GemWalletHomeServiceMock(),
        onManageAssets: @escaping () -> Void = {},
    ) -> NetworkAssetsSceneViewModel {
        NetworkAssetsSceneViewModel(
            wallet: .mock(),
            chain: .ethereum,
            service: service,
            onManageAssets: onManageAssets,
        )
    }
}
