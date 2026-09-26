// Copyright (c). Gem Wallet. All rights reserved.

import Primitives
@testable import PrimitivesComponents
import PrimitivesTestKit
import Testing

struct AssetsSectionsTests {
    @Test
    func pinnedAssetsStaySeparateFromPopularAndAssets() {
        let sections = AssetsSections.from(
            [
                .mock(asset: .mock(id: .mock(chain: .smartChain)), metadata: .mock(isPinned: true)),
                .mock(asset: .mock(id: .mock(chain: .ethereum)), metadata: .mock(isPinned: false)),
                .mock(asset: .mock(id: .mock(chain: .tron)), metadata: .mock(isPinned: false)),
            ],
            showsPopular: true,
        )

        #expect(sections.pinned.map(\.asset.id.chain) == [.smartChain])
        #expect(sections.popular.map(\.asset.id.chain) == [.ethereum])
        #expect(sections.assets.map(\.asset.id.chain) == [.tron])
    }
}
