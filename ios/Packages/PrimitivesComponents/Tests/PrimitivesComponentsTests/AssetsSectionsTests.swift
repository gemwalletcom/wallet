// Copyright (c). Gem Wallet. All rights reserved.

import Primitives
import PrimitivesTestKit
@testable import PrimitivesComponents
import Testing

struct AssetsSectionsTests {
    @Test
    func popularEnabledRemovesPopularFromAssets() {
        let sections = AssetsSections.from(
            [
                .mock(asset: .mock(id: .mock(.bitcoin)), metadata: .mock(isPinned: false)),
                .mock(asset: .mock(id: .mock(.ethereum)), metadata: .mock(isPinned: false)),
                .mock(asset: .mock(id: .mock(.solana)), metadata: .mock(isPinned: false)),
                .mock(asset: .mock(id: .mock(.smartChain)), metadata: .mock(isPinned: false)),
            ],
            showsPopular: true
        )

        #expect(sections.popular.map { $0.asset.id.chain } == [.bitcoin, .ethereum, .solana])
        #expect(sections.assets.map { $0.asset.id.chain } == [.smartChain])
    }

    @Test
    func popularDisabledKeepsPopularInAssets() {
        let sections = AssetsSections.from([
            .mock(asset: .mock(id: .mock(.bitcoin)), metadata: .mock(isPinned: false)),
            .mock(asset: .mock(id: .mock(.smartChain)), metadata: .mock(isPinned: false)),
        ])

        #expect(sections.popular.isEmpty)
        #expect(sections.assets.map { $0.asset.id.chain } == [.bitcoin, .smartChain])
    }

    @Test
    func pinnedAssetsStaySeparateFromPopularAndAssets() {
        let sections = AssetsSections.from(
            [
                .mock(asset: .mock(id: .mock(.smartChain)), metadata: .mock(isPinned: true)),
                .mock(asset: .mock(id: .mock(.ethereum)), metadata: .mock(isPinned: false)),
                .mock(asset: .mock(id: .mock(.tron)), metadata: .mock(isPinned: false)),
            ],
            showsPopular: true
        )

        #expect(sections.pinned.map { $0.asset.id.chain } == [.smartChain])
        #expect(sections.popular.map { $0.asset.id.chain } == [.ethereum])
        #expect(sections.assets.map { $0.asset.id.chain } == [.tron])
    }
}
