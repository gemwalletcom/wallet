// Copyright (c). Gem Wallet. All rights reserved.

import Primitives
import PrimitivesTestKit
@testable import PrimitivesComponents
import Testing

struct AssetsSectionsTests {
    private func asset(_ chain: Chain, isPinned: Bool = false) -> AssetData {
        AssetData.mock(
            asset: .mock(id: AssetId(chain: chain)),
            metadata: .mock(isPinned: isPinned)
        )
    }

    @Test
    func popularEnabledRemovesPopularFromAssets() {
        let sections = AssetsSections.from(
            [asset(.bitcoin), asset(.ethereum), asset(.solana), asset(.smartChain)],
            popularIds: [Chain.bitcoin.assetId, Chain.ethereum.assetId, Chain.solana.assetId]
        )

        #expect(sections.popular.map { $0.asset.id.chain } == [.bitcoin, .ethereum, .solana])
        #expect(sections.assets.map { $0.asset.id.chain } == [.smartChain])
    }

    @Test
    func popularDisabledKeepsPopularInAssets() {
        let sections = AssetsSections.from([asset(.bitcoin), asset(.smartChain)])

        #expect(sections.popular.isEmpty)
        #expect(sections.assets.map { $0.asset.id.chain } == [.bitcoin, .smartChain])
    }

    @Test
    func pinnedAssetsStaySeparateFromPopularAndAssets() {
        let sections = AssetsSections.from(
            [asset(.smartChain, isPinned: true), asset(.ethereum), asset(.tron)],
            popularIds: [Chain.bitcoin.assetId, Chain.ethereum.assetId, Chain.solana.assetId]
        )

        #expect(sections.pinned.map { $0.asset.id.chain } == [.smartChain])
        #expect(sections.popular.map { $0.asset.id.chain } == [.ethereum])
        #expect(sections.assets.map { $0.asset.id.chain } == [.tron])
    }
}
