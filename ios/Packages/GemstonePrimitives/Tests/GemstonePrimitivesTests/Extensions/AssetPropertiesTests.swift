// Copyright (c). Gem Wallet. All rights reserved.

@testable import GemstonePrimitives
import Primitives
import Testing

struct AssetPropertiesTests {
    @Test func defaultValueIsStakeable() {
        #expect(AssetProperties.defaultValue(assetId: AssetId(chain: .smartChain)).isSwapable == true)
        #expect(AssetProperties.defaultValue(assetId: AssetId(chain: .smartChain, tokenId: "0x")).isStakeable == false)
        #expect(AssetProperties.defaultValue(assetId: .mockSolana()).isStakeable == true)
        #expect(AssetProperties.defaultValue(assetId: .mockSolanaUSDC()).isStakeable == false)
    }

    @Test func defaultValueIsSwapable() {
        #expect(AssetProperties.defaultValue(assetId: AssetId(chain: .smartChain)).isSwapable == true)
        #expect(AssetProperties.defaultValue(assetId: AssetId(chain: .smartChain, tokenId: "0x")).isSwapable == true)
        #expect(AssetProperties.defaultValue(assetId: .mockSolana()).isSwapable == true)
        #expect(AssetProperties.defaultValue(assetId: .mockSolanaUSDC()).isSwapable == true)
    }
}
