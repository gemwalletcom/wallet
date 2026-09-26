// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Components
import enum Gemstone.GemConfirmHeader
import struct Gemstone.GemSimulationValue
import GemstonePrimitives
import GemstonePrimitivesTestKit
@testable import Primitives
import PrimitivesComponents
import PrimitivesComponentsTestKit
import PrimitivesTestKit
import Testing
@testable import Transfer
@testable import TransferTestKit

struct ConfirmHeaderItemTests {
    @Test
    func amountShowsClearHeader() {
        let headerType = TransactionHeaderType.amount(.numeric(.mock()))
        #expect(headerType.showsClearHeader == true)
    }

    @Test
    func swapHidesClearHeader() {
        let headerType = TransactionHeaderType.swap(
            from: SwapAmountField(
                assetId: .mock(chain: .ethereum),
                assetImage: AssetImage(),
                amount: "1 ETH",
                fiatAmount: "$1",
            ),
            to: SwapAmountField(
                assetId: Asset.mock(id: .mock(chain: .ethereum, tokenId: "0xdAC17F958D2ee523a2206206994597C13D831ec7"), name: "Tether", symbol: "USDT", decimals: 6, type: .erc20).id,
                assetImage: AssetImage(),
                amount: "2 USDC",
                fiatAmount: "$2",
            ),
        )
        #expect(headerType.showsClearHeader == false)
    }

    @Test
    func nftShowsClearHeader() {
        #expect(TransactionHeaderType.nft(name: nil, image: AssetImage()).showsClearHeader == true)
    }

    @Test
    func assetShowsClearHeader() {
        #expect(TransactionHeaderType.asset(image: AssetImage()).showsClearHeader == true)
    }

    @Test
    func aValueHeaderDrawsTheAssetAndWhatItApproves() {
        let value = GemSimulationValue(
            asset: Asset.mock(id: .mock(chain: .ethereum, tokenId: "0xdAC17F958D2ee523a2206206994597C13D831ec7"), name: "Tether", symbol: "USDT", decimals: 6, type: .erc20).toGem(),
            value: .exact(value: BigUInt(1_000_000)),
            icon: .mock(),
        )
        let model = GemConfirmHeader.value(value: value)

        guard case let .header(headerType, _) = model.itemModel,
              case let .assetValue(header) = headerType
        else {
            Issue.record("Expected assetValue header")
            return
        }
        #expect(header.title == "1 USDT")
        #expect(header.assetImage != nil)
        #expect(headerType.showsClearHeader)
    }

    @Test
    func aPlaceholderKeepsTheHeadInPlaceUntilTheValueArrives() {
        let model = GemConfirmHeader.placeholder(icon: .mock())

        guard case let .header(headerType, _) = model.itemModel,
              case let .assetValue(header) = headerType
        else {
            Issue.record("Expected assetValue header")
            return
        }
        #expect(header.title.isEmpty)
        #expect(header.assetImage != nil)
        #expect(headerType.showsClearHeader)
    }

    @Test
    func aTransactionHeaderReadsThroughTheSharedMapper() {
        let model = GemConfirmHeader.transaction(header: .amount(amount: .mock(asset: Primitives.Asset.mock(
            id: .mock(chain: .ethereum, tokenId: "0xdAC17F958D2ee523a2206206994597C13D831ec7"),
            name: "Tether",
            symbol: "USDT",
            decimals: 6,
            type: .erc20,
        ).toGem())))

        guard case let .header(headerType, _) = model.itemModel, case .amount = headerType else {
            Issue.record("Expected the amount header")
            return
        }
        #expect(headerType.showsClearHeader)
    }

    @Test
    func aReservedHeaderKeepsItsPlaceHidden() {
        let model = GemConfirmHeader.reserved(header: .amount(amount: .mock(asset: Primitives.Asset.mock(
            id: .mock(chain: .ethereum, tokenId: "0xdAC17F958D2ee523a2206206994597C13D831ec7"),
            name: "Tether",
            symbol: "USDT",
            decimals: 6,
            type: .erc20,
        ).toGem())))

        guard case let .header(headerType, isReserved) = model.itemModel, case .amount = headerType else {
            Issue.record("Expected the amount header")
            return
        }
        #expect(isReserved)
    }
}
