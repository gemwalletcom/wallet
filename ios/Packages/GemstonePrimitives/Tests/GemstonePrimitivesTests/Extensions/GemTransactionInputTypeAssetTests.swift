// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.GemTransactionInputType
import GemstonePrimitives
import Primitives
import PrimitivesTestKit
import Testing

struct GemTransactionInputTypeAssetTests {
    @Test
    func localAssetMatchesTheCoreRuleForEveryCase() {
        for type in cases {
            let fromCore = type.transactionAsset().map()

            #expect(type.asset == fromCore, "asset disagreed with Core for \(type)")
        }
    }

    @Test
    func localAssetIsTheNativeAssetForAnNftTransfer() {
        let nftAsset = NFTAsset.mock()

        #expect(GemTransactionInputType.transferNft(nftAsset).asset == nftAsset.chain.asset)
    }

    private var cases: [GemTransactionInputType] {
        let asset = Asset.mock()
        return [
            .transfer(asset),
            .deposit(asset),
            .withdrawal(asset),
            .swap(asset, .mockEthereum(), .mock()),
            .stake(asset, .rewards([])),
            .tokenApprove(asset, .mock()),
            .transferNft(.mock()),
        ]
    }
}
