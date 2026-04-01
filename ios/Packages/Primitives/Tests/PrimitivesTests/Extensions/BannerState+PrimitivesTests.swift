// Copyright (c). Gem Wallet. All rights reserved.

@testable import Primitives
import Testing

struct BannerStateComparableTests {
    @Test
    func bannerStateSortPriority() {
        let banners = [
            BannerState.cancelled,
            BannerState.active,
            BannerState.alwaysActive,
        ]

        let sorted = banners.sorted { $0 < $1 }

        #expect(sorted[0] == .alwaysActive)
        #expect(sorted[1] == .active)
        #expect(sorted[2] == .cancelled)
    }
}
