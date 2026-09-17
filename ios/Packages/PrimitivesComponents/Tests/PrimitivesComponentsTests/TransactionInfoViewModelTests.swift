// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Primitives
@testable import PrimitivesComponents
import PrimitivesComponentsTestKit
import PrimitivesTestKit
import Testing

struct TransactionInfoModelTests {
    let asset = Asset.mock()
    let feeAsset = Asset.mock()

    @Test
    func amountDisplay() {
        let model = TransactionInfoViewModel.mock(sign: .incoming)

        let display = model.amountDisplay()
        #expect(display.amount.text.contains(asset.symbol))
        #expect(!display.amount.text.isEmpty)
        #expect(display.amount.text == "+1 BTC")
    }

    @Test
    func amountDisplayOutgoing() {
        let model = TransactionInfoViewModel.mock(sign: .outgoing)

        let display = model.amountDisplay()
        #expect(display.amount.text.contains(asset.symbol))
        #expect(!display.amount.text.isEmpty)
        #expect(display.amount.text == "-1 BTC")
    }

    @Test
    func amountDisplayFiat() {
        let model = TransactionInfoViewModel.mock()

        let display = model.amountDisplay()
        #expect(display.fiat != nil)
        #expect(display.fiat?.text == "$1.50")
    }

    @Test
    func feeDisplay() throws {
        let model = TransactionInfoViewModel.mock(sign: .incoming)

        let feeDisplay = try #require(model.feeDisplay)
        #expect(feeDisplay.amount.text.contains(feeAsset.symbol))
        #expect(feeDisplay.amount.text == "0.1 BTC")
    }

    @Test
    func feeDisplayFiat() throws {
        let model = TransactionInfoViewModel.mock()

        #expect(model.feeDisplay != nil)
        #expect(try #require(model.feeDisplay?.fiat) != nil)
        #expect(model.feeDisplay?.fiat?.text == "$0.05")
    }

    @Test
    func headerTypeAmount() {
        let model = TransactionInfoViewModel.mock(sign: .incoming)
        let header = model.headerType(input: .amount(showFiat: true))
        guard case let .amount(display) = header else {
            Issue.record("Expected header type .amount")
            return
        }

        #expect(display.amount.text == "+1 BTC")
        #expect(display.fiat?.text == "$1.50")
    }

    @Test
    func headerTypeNFT() {
        let nftAsset = NFTAsset.mock()
        let model = TransactionInfoViewModel.mock()

        let header = model.headerType(input: .nft(name: nftAsset.name, id: nftAsset.id.identifier))
        guard case let .nft(name, _) = header else {
            Issue.record("Expected header type .nft")
            return
        }

        #expect(name == nftAsset.name)
    }

    @Test
    func headerTypeSwap() {
        let swapMetadata = SwapHeaderInput(
            from: .mock(asset: asset, price: .mock(price: 1.5)),
            to: .mock(asset: feeAsset, value: BigInt(10_000_000), price: .mock(price: 0.5)),
        )

        let model = TransactionInfoViewModel.mock()

        let header = model.headerType(input: .swap(swapMetadata))
        guard case let .swap(fromField, toField) = header else {
            Issue.record("Expected header type .swap")
            return
        }

        #expect(fromField.amount.contains(asset.symbol))
        #expect(toField.amount.contains(feeAsset.symbol))
        #expect(fromField.assetId == asset.id)
        #expect(toField.assetId == feeAsset.id)
        #expect(fromField.amount == "1 BTC")
        #expect(toField.amount == "0.1 BTC")
    }

    @Test
    func amountDisplayFiatNilWhenAssetPriceIsNil() {
        let model = TransactionInfoViewModel.mock(assetPrice: nil)

        let display = model.amountDisplay()
        #expect(display.fiat == nil)
    }

    @Test
    func feeDisplayNilWhenFeeValueIsNil() {
        let model = TransactionInfoViewModel.mock(feeValue: nil)

        #expect(model.feeDisplay == nil)
    }

    @Test
    func feeDisplayFiatNilWhenFeeAssetPriceIsNil() {
        let model = TransactionInfoViewModel.mock(feeAssetPrice: nil)

        #expect(model.feeDisplay?.fiat == nil)
    }

    @Test
    func headerTypeAmountWithoutFiat() {
        let model = TransactionInfoViewModel.mock(sign: .incoming)
        let header = model.headerType(input: .amount(showFiat: false))
        guard case let .amount(display) = header else {
            Issue.record("Expected header type .amount")
            return
        }

        #expect(display.amount.text == "+1 BTC")
        #expect(display.fiat == nil)
    }
}
