// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.GemTransactionInputType
import class Gemstone.GemTransferService
import GemstonePrimitives
import Primitives
import PrimitivesTestKit
import Testing

struct GemTransactionInputTypeAssetTests {
    private let transferService = GemTransferService()

    @Test
    func localAssetMatchesTheCoreRuleForEveryCase() {
        for type in cases {
            let fromCore = transferService.asset(inputType: type).map()

            #expect(type.asset == fromCore, "asset disagreed with Core for \(type)")
        }
    }

    @Test
    func localAssetIsTheNativeAssetForAnNftTransfer() {
        let nftAsset = NFTAsset.mock()

        #expect(GemTransactionInputType.transferNft(nftAsset).asset == Asset(nftAsset.chain))
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
