// Copyright (c). Gem Wallet. All rights reserved.

@testable import GemstonePrimitives
import Primitives
import PrimitivesTestKit
import Testing

final class AssetTests {
    let nativeAsset = Asset(.ethereum)
    let tokenAsset = Asset(id: AssetId(chain: .ethereum, tokenId: "0x123"), name: "", symbol: "", decimals: 18, type: .erc20)

    @Test
    func assetFee() {
        #expect(nativeAsset.feeAsset == nativeAsset)
        #expect(tokenAsset.feeAsset != tokenAsset)
        #expect(tokenAsset.feeAsset == nativeAsset)
    }

    @Test
    func feeAssetHypercorePerpetual() {
        let perpetual = Asset.hypercoreUSDC()
        #expect(perpetual.feeAsset == Asset.hypercoreUSDC())
    }

    @Test
    func feeAssetHypercoreToken() {
        let token = Asset.hypercoreSpotUSDC()
        #expect(token.feeAsset == Asset.hypercoreSpotUSDC())
    }

    @Test
    func feeAssetHypercoreNative() {
        let native = Asset(.hyperCore)
        #expect(native.feeAsset == Asset.hypercoreSpotUSDC())
    }

    @Test
    func feeAssetUsesSentAssetOnlyWhenFeeAssetIdMatches() {
        #expect(tokenAsset.feeAsset(for: Fee.mock(feeAssetId: tokenAsset.id)) == tokenAsset)
        #expect(tokenAsset.feeAsset(for: Fee.mock(feeAssetId: tokenAsset.feeAsset.id)) == tokenAsset.feeAsset)
        #expect(tokenAsset.feeAsset(for: Fee.mock(feeAssetId: AssetId(chain: .bitcoin))) == tokenAsset.feeAsset)
    }
}
