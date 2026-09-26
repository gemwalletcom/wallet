// Copyright (c). Gem Wallet. All rights reserved.

import Components
import struct Gemstone.GemTransactionRow
import GemstonePrimitives
import GemstonePrimitivesTestKit
import Primitives
import PrimitivesComponents
import PrimitivesComponentsTestKit
import PrimitivesTestKit
import Testing

final class GemTransactionRowTests {
    @Test
    func transactionTitle() {
        #expect(GemTransactionRow.mock(type: .transfer, state: .confirmed).titleTextValue.text == "Received")
        #expect(GemTransactionRow.mock(type: .transfer, state: .confirmed, direction: .outgoing).titleTextValue.text == "Sent")
        #expect(GemTransactionRow.mock(type: .transfer, state: .failed).titleTextValue.text == "Transfer")
        #expect(GemTransactionRow.mock(type: .swap).titleTextValue.text == "Swap")
    }

    @Test
    func autoValueFormatter() {
        let fromAsset = Asset.mock(id: .mock(chain: .ethereum), name: "Ethereum", symbol: "ETH", decimals: 18)
        let toAsset = Asset.mock(id: .mock(chain: .ethereum, tokenId: "0xdAC17F958D2ee523a2206206994597C13D831ec7"), name: "Tether", symbol: "USDT", decimals: 6, type: .erc20)
        #expect(GemTransactionRow.mock(metadata: .encode(TransactionSwapMetadata.mock(fromAsset: fromAsset.id, toAsset: toAsset.id, toValue: "1000000"))).subtitleTextValue?.text == "+1 USDT")
        #expect(GemTransactionRow.mock(metadata: .encode(TransactionSwapMetadata.mock(fromAsset: fromAsset.id, toAsset: toAsset.id, toValue: "10000"))).subtitleTextValue?.text == "+0.01 USDT")
        #expect(GemTransactionRow.mock(metadata: .encode(TransactionSwapMetadata.mock(fromAsset: fromAsset.id, toAsset: toAsset.id, toValue: "1000"))).subtitleTextValue?.text == "+0.001 USDT")
        #expect(GemTransactionRow.mock(metadata: .encode(TransactionSwapMetadata.mock(fromAsset: fromAsset.id, toAsset: toAsset.id, toValue: "100"))).subtitleTextValue?.text == "+0.0001 USDT")
        #expect(GemTransactionRow.mock(metadata: .encode(TransactionSwapMetadata.mock(fromAsset: fromAsset.id, toAsset: toAsset.id, toValue: "10"))).subtitleTextValue?.text == "+<0.0001 USDT")
        #expect(GemTransactionRow.mock(metadata: .encode(TransactionSwapMetadata.mock(fromAsset: fromAsset.id, toAsset: toAsset.id, toValue: "1"))).subtitleTextValue?.text == "+<0.0001 USDT")
    }

    @Test
    func titleExtraUsesAddressNames() {
        let toAddress = AddressName.mock(address: "0x742d35cc6327c516e07e17dddaef8b48ca1e8c4a", name: "Hyperliquid")
        let hyperliquidViewModel = GemTransactionRow.mock(
            type: .transfer,
            direction: .outgoing,
            to: "0x742d35cc6327c516e07e17dddaef8b48ca1e8c4a",
            toAddress: toAddress,
            metadata: .encode(TransactionSwapMetadata.mock()),
        )
        #expect(hyperliquidViewModel.titleExtraTextValue?.text.contains("Hyperliquid") == true)

        let fromAddress = AddressName.mock(address: "0x1111111111111111111111111111111111111111", name: "Sender")
        let incomingViewModel = GemTransactionRow.mock(
            type: .transfer,
            direction: .incoming,
            from: "0x1111111111111111111111111111111111111111",
            fromAddress: fromAddress,
            metadata: .encode(TransactionSwapMetadata.mock()),
        )
        #expect(incomingViewModel.titleExtraTextValue?.text.contains("Sender") == true)

        let unknownViewModel = GemTransactionRow.mock(
            type: .transfer,
            direction: .outgoing,
            to: "0x1234567890abcdef1234567890abcdef12345678",
            metadata: .encode(TransactionSwapMetadata.mock()),
        )
        #expect(unknownViewModel.titleExtraTextValue?.text.contains("0x1234") == true)
        #expect(unknownViewModel.titleExtraTextValue?.text.contains("5678") == true)
    }

    @Test
    func titleExtraPerpetualShowsPriceWithLabel() {
        let openPositionModel = GemTransactionRow.mock(
            type: .perpetualOpenPosition,
            asset: Asset.mock(id: .mock(chain: .hyperCore, tokenId: "perpetual::USDC"), name: "USDC", symbol: "USDC", decimals: 6, type: .perpetual),
            metadata: perpetualMetadata(price: 50000.50),
        )
        #expect(openPositionModel.titleExtraTextValue?.text == "Price: $50,000.50")

        let closePositionModel = GemTransactionRow.mock(
            type: .perpetualClosePosition,
            asset: Asset.mock(id: .mock(chain: .hyperCore, tokenId: "perpetual::USDC"), name: "USDC", symbol: "USDC", decimals: 6, type: .perpetual),
            metadata: perpetualMetadata(price: 49999.99),
        )
        #expect(closePositionModel.titleExtraTextValue?.text == "Price: $49,999.99")
    }

    @Test
    func titleExtraPerpetualHidesMissingPrice() {
        let model = GemTransactionRow.mock(
            type: .perpetualClosePosition,
            state: .pending,
            asset: Asset.mock(id: .mock(chain: .hyperCore, tokenId: "perpetual::USDC"), name: "USDC", symbol: "USDC", decimals: 6, type: .perpetual),
            metadata: perpetualMetadata(price: 0),
        )

        #expect(model.titleExtraTextValue == nil)
    }

    @Test
    func subtitlePerpetualOpenPositionShowsFiatValue() {
        let model = GemTransactionRow.mock(
            type: .perpetualOpenPosition,
            value: "1000000",
            asset: Asset.mock(id: .mock(chain: .hyperCore, tokenId: "perpetual::USDC"), name: "USDC", symbol: "USDC", decimals: 6, type: .perpetual),
            metadata: perpetualMetadata(),
        )
        #expect(model.subtitleTextValue?.text == "$1.00")
    }

    @Test
    func subtitlePerpetualClosePositionShowsPnl() {
        let profitModel = GemTransactionRow.mock(
            type: .perpetualClosePosition,
            asset: Asset.mock(id: .mock(chain: .hyperCore, tokenId: "perpetual::USDC"), name: "USDC", symbol: "USDC", decimals: 6, type: .perpetual),
            metadata: perpetualMetadata(pnl: 125.50),
        )
        #expect(profitModel.subtitleTextValue?.text == "+$125.50")

        let lossModel = GemTransactionRow.mock(
            type: .perpetualClosePosition,
            asset: Asset.mock(id: .mock(chain: .hyperCore, tokenId: "perpetual::USDC"), name: "USDC", symbol: "USDC", decimals: 6, type: .perpetual),
            metadata: perpetualMetadata(pnl: -75.25),
        )
        #expect(lossModel.subtitleTextValue?.text == "-$75.25")
    }

    @Test
    func subtitlePerpetualClosePositionNoPnl() {
        let model = GemTransactionRow.mock(
            type: .perpetualClosePosition,
            asset: Asset.mock(id: .mock(chain: .hyperCore, tokenId: "perpetual::USDC"), name: "USDC", symbol: "USDC", decimals: 6, type: .perpetual),
            metadata: perpetualMetadata(pnl: 0),
        )
        #expect(model.subtitleTextValue == nil)
    }

    @Test
    func titleExtraFreeze() {
        let bandwidthModel = GemTransactionRow.mock(
            type: .stakeFreeze,
            metadata: .object(["resourceType": .string("bandwidth")]),
        )
        #expect(bandwidthModel.titleExtraTextValue?.text == "To Bandwidth")

        let energyModel = GemTransactionRow.mock(
            type: .stakeFreeze,
            metadata: .object(["resourceType": .string("energy")]),
        )
        #expect(energyModel.titleExtraTextValue?.text == "To Energy")
    }

    @Test
    func titleExtraUnfreeze() {
        let bandwidthModel = GemTransactionRow.mock(
            type: .stakeUnfreeze,
            metadata: .object(["resourceType": .string("bandwidth")]),
        )
        #expect(bandwidthModel.titleExtraTextValue?.text == "From Bandwidth")

        let energyModel = GemTransactionRow.mock(
            type: .stakeUnfreeze,
            metadata: .object(["resourceType": .string("energy")]),
        )
        #expect(energyModel.titleExtraTextValue?.text == "From Energy")
    }

    @Test
    func titleExtraFreezeNoMetadata() {
        let model = GemTransactionRow.mock(
            type: .stakeFreeze,
            metadata: .null,
        )
        #expect(model.titleExtraTextValue == nil)
    }

    @Test
    func titleExtraHidesEmptyParticipant() {
        let model = GemTransactionRow.mock(type: .stakeDelegate, direction: .outgoing, to: "")

        #expect(model.titleExtraTextValue == nil)
    }

    @Test
    func titleExtraHidesEmptySender() {
        let model = GemTransactionRow.mock(type: .transfer, direction: .incoming, from: "", to: "0x123")

        #expect(model.titleExtraTextValue == nil)
    }

    @Test
    func titleTagShowsProgressForActiveStates() {
        let pendingModel = GemTransactionRow.mock(state: .pending)
        if case .progressView = pendingModel.titleTagType {
        } else {
            Issue.record("Expected progress indicator for pending title tag")
        }
        #expect(pendingModel.titleTagTextValue?.text == TransactionStateViewModel(state: .pending, tone: .pending).title)

        let inTransitModel = GemTransactionRow.mock(state: .inTransit)
        if case .progressView = inTransitModel.titleTagType {
        } else {
            Issue.record("Expected progress indicator for in-transit title tag")
        }
        #expect(inTransitModel.titleTagTextValue?.text == TransactionStateViewModel(state: .inTransit, tone: .pending).title)
    }

    private func perpetualMetadata(pnl: Double = 0, price: Double = 0) -> AnyCodableValue {
        .object(["pnl": .double(pnl), "price": .double(price), "direction": .string(PerpetualDirection.short.rawValue)])
    }
}
