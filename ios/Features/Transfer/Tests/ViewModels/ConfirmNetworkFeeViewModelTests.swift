// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.GemFeeText
import GemstonePrimitivesTestKit
import Primitives
import PrimitivesComponents
import PrimitivesComponentsTestKit
import PrimitivesTestKit
import Testing
@testable import Transfer

struct ConfirmNetworkFeeViewModelTests {
    @Test
    func readyPrintsTheFeeTextCoreBuilt() {
        let text = GemFeeText.mock(value: .mock(value: 0.0001446, unit: .symbol(symbol: "BNB")), extra: .text(text: "$0.11"))
        let model = ConfirmNetworkFeeViewModel(
            feeRow: .ready(text: text),
            feeModel: .mock(feeRates: .mock(rows: [], showsOptions: false, unitType: .gwei, unitDecimals: 9, supportsCustomFee: false, selectedTotal: nil, normalTotal: nil, customRate: nil)),
            infoAction: {},
        )

        guard case let .networkFee(item, selectable) = model.itemModel else {
            Issue.record("Expected network fee item")
            return
        }
        #expect(item.subtitle == text.value.text())
        #expect(item.subtitleExtra == "$0.11")
        #expect(selectable)
    }

    @Test
    func error() {
        let pathUSD = FeeAssetItem.mock(asset: .mock(id: .mock(chain: .tempo, tokenId: "0x20C0000000000000000000000000000000000000"), name: "pathUSD", symbol: "pathUSD", decimals: 6, type: .tip20))
        let usdc = FeeAssetItem.mock(asset: .mock(id: .mock(chain: .tempo, tokenId: "0x20C000000000000000000000b9537d11c60E8b50"), name: "Bridged USDC", symbol: "USDC.e", decimals: 6, type: .tip20))
        let model = ConfirmNetworkFeeViewModel(
            feeRow: .unavailable(text: "-"),
            feeModel: .mock(
                feeAsset: pathUSD.asset,
                feeAssetPrice: .mock(price: 1),
                feeAmount: 1,
                feeAssets: [pathUSD, usdc],
                showsFeeAssets: true,
                onSelectFeeAsset: { _ in },
            ),
            infoAction: {},
        )

        guard case let .networkFee(item, selectable) = model.itemModel else {
            Issue.record("Expected network fee item")
            return
        }
        #expect(item.subtitle == "-")
        #expect(item.subtitleExtra == nil)
        #expect(selectable == false)
    }
}
