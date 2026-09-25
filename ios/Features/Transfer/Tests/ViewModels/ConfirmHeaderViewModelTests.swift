// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Components
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

struct ConfirmHeaderViewModelTests {
    @Test
    func amountShowsClearHeader() {
        let headerType = TransactionHeaderType.amount(.numeric(.mock()))
        #expect(headerType.showsClearHeader == true)
    }

    @Test
    func swapHidesClearHeader() {
        let headerType = TransactionHeaderType.swap(
            from: SwapAmountField(
                assetId: .mockEthereum(),
                assetImage: AssetImage(),
                amount: "1 ETH",
                fiatAmount: "$1",
            ),
            to: SwapAmountField(
                assetId: Asset.mockEthereumUSDT().id,
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
        let value = GemSimulationValue(asset: Asset.mockEthereumUSDT().toGem(), value: .exact(value: BigUInt(1_000_000)))
        let model = ConfirmHeaderViewModel(header: .value(value: value))

        guard case let .header(headerType, _) = model.itemModel,
              case let .assetValue(header) = headerType,
              let data = header as? AssetValueHeaderViewModel
        else {
            Issue.record("Expected assetValue header")
            return
        }
        #expect(data.data.asset == Asset.mockEthereumUSDT().toGem())
        #expect(data.data.value == .exact(value: BigUInt(1_000_000)))
        #expect(headerType.showsClearHeader)
    }

    @Test
    func aPlaceholderKeepsTheHeadInPlaceUntilTheValueArrives() {
        let model = ConfirmHeaderViewModel(header: .placeholder(assetId: Asset.mockEthereumUSDT().id.identifier))

        guard case let .header(headerType, _) = model.itemModel,
              case let .assetValue(header) = headerType
        else {
            Issue.record("Expected assetValue header")
            return
        }
        #expect(header is AssetValueHeaderPlaceholder)
        #expect(headerType.showsClearHeader)
    }

    @Test
    func aTransactionHeaderReadsThroughTheSharedMapper() {
        let model = ConfirmHeaderViewModel(header: .transaction(header: .amount(amount: .mock(asset: .mockEthereumUSDT()))))

        guard case let .header(headerType, _) = model.itemModel, case .amount = headerType else {
            Issue.record("Expected the amount header")
            return
        }
        #expect(headerType.showsClearHeader)
    }

    @Test
    func aReservedHeaderKeepsItsPlaceHidden() {
        let model = ConfirmHeaderViewModel(header: .reserved(header: .amount(amount: .mock(asset: .mockEthereumUSDT()))))

        guard case let .header(headerType, isReserved) = model.itemModel, case .amount = headerType else {
            Issue.record("Expected the amount header")
            return
        }
        #expect(isReserved)
    }
}
