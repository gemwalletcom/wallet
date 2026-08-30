// Copyright (c). Gem Wallet. All rights reserved.

import class Gemstone.GemTransferService
import GemstonePrimitives
import Primitives
import PrimitivesTestKit
import Testing

struct TransferDataTypeAssetTests {
    private let transferService = GemTransferService()

    @Test
    func localAssetMatchesTheCoreRuleForEveryCase() throws {
        for type in cases {
            let fromCore = try Asset(transferService.asset(inputType: type.inputType))

            #expect(type.asset == fromCore, "asset disagreed with Core for \(type)")
        }
    }

    @Test
    func localAssetIsTheNativeAssetForAnNftTransfer() {
        let nftAsset = NFTAsset.mock()

        #expect(TransferDataType.transferNft(nftAsset).asset == Asset(nftAsset.chain))
    }

    private var cases: [TransferDataType] {
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
