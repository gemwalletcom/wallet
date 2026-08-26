// Copyright (c). Gem Wallet. All rights reserved.

import GemstoneServicesTestKit
import Foundation
import Primitives
import Recents
import StoreTestKit

public extension RecentsSceneViewModel {
    static func mock(
        walletId: WalletId = .mock(),
        types: [RecentActivityType] = [],
        onSelect: @escaping (Asset) -> Void = { _ in },
    ) -> RecentsSceneViewModel {
        RecentsSceneViewModel(
            walletId: walletId,
            types: types,
            recentActivityStore: .mock(),
            onSelect: onSelect,
        )
    }
}
