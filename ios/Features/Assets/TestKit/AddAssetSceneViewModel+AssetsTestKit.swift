// Copyright (c). Gem Wallet. All rights reserved.

import Assets
import GemstoneServicesTestKit
import Primitives
import PrimitivesTestKit

public extension AddAssetSceneViewModel {
    @MainActor
    static func mock() -> AddAssetSceneViewModel {
        AddAssetSceneViewModel(
            wallet: .mock(accounts: [.mock(chain: .ethereum)]),
            service: GemAddAssetServiceMock(),
        )
    }
}
