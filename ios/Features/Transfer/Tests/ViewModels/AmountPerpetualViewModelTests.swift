// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import struct Gemstone.GemAmountInput
import class Gemstone.GemAmountService
import struct Gemstone.GemAssetBalance
import struct Gemstone.GemPerpetualAutoclose
import struct Gemstone.GemTransferData
import enum Gemstone.PerpetualType
import GemstonePrimitivesTestKit
import GemstoneServicesTestKit
import Primitives
import PrimitivesTestKit
import Testing
@testable import Transfer
import TransferTestKit

struct AmountPerpetualViewModelTests {
    @Test
    func title() {
        let openLong = AmountPerpetualViewModel.mock(action: .open(data: .mock(direction: .long, price: 100, leverage: 3)))
        let openShort = AmountPerpetualViewModel.mock(action: .open(data: .mock(direction: .short, price: 100, leverage: 3)))

        #expect(amountTitle(openLong) == "Long")
        #expect(amountTitle(openShort) == "Short")
    }

    @Test
    func increaseReduceTitle() {
        let increase = AmountPerpetualViewModel.mock(action: .increase(data: .mock(direction: .long)))
        let reduce = AmountPerpetualViewModel.mock(action: .reduce(data: .mock(direction: .long, price: 100, leverage: 3), position: PerpetualPosition.mock(marginAmount: 0.001).toGem()))

        #expect(amountTitle(increase).contains("Long"))
        #expect(amountTitle(reduce).contains("Long"))
    }

    @Test
    func leverageSelection() {
        let open = AmountPerpetualViewModel.mock(action: .open(data: .mock(direction: .long, price: 100, leverage: 10)))
        let increase = AmountPerpetualViewModel.mock(action: .increase(data: .mock(direction: .long, price: 100, leverage: 3)))

        #expect(open.leverageSelection != nil)
        #expect(open.leverageSelection?.isEnabled == true)
        #expect(increase.leverageSelection == nil)
    }

    @Test
    func isAutocloseEnabled() {
        let open = AmountPerpetualViewModel.mock()
        let increase = AmountPerpetualViewModel.mock(action: .increase(data: .mock(direction: .long, price: 100, leverage: 3)))
        let reduce = AmountPerpetualViewModel.mock(action: .reduce(data: .mock(direction: .long, price: 100, leverage: 3), position: PerpetualPosition.mock(marginAmount: 0.001).toGem()))

        #expect(open.isAutocloseEnabled == true)
        #expect(increase.isAutocloseEnabled == false)
        #expect(reduce.isAutocloseEnabled == false)
    }

    @Test
    func availableValue() {
        let assetData = AssetData.mock(balance: .mock(available: 5000))

        let open = AmountPerpetualViewModel.mock()
        let reduce = AmountPerpetualViewModel.mock(action: .reduce(data: .mock(direction: .long, price: 100, leverage: 3), position: PerpetualPosition.mock(marginAmount: 0.001).toGem()))

        #expect(amountInput(open, assetData).availableValue == 5000)
        #expect(amountInput(reduce, assetData).availableValue == 1000)
    }

    @Test
    func autocloseListItem() {
        let model = AmountPerpetualViewModel.mock()

        #expect(model.autocloseListItem?.subtitle == "-")
        #expect(model.autocloseListItem?.subtitleExtra == nil)

        model.updateAutoclose(takeProfit: "100", stopLoss: nil)
        #expect(model.autocloseListItem?.subtitle == "TP: $100.00")

        model.updateAutoclose(takeProfit: "100", stopLoss: "50")
        #expect(model.autocloseListItem?.subtitleExtra == "SL: $50.00")
    }

    @Test
    func aLeverageChangeRefreshesUntouchedDefaultsAndKeepsEditedPrices() throws {
        let service = GemAmountServiceMock(builder: GemAmountService.mock())
        service.perpetualAutocloseValue = { leverage in GemPerpetualAutoclose(takeProfit: "\(100 + Int(leverage))", stopLoss: "\(50 - Int(leverage))") }
        let model = AmountPerpetualViewModel.mock(action: .open(data: .mock(direction: .long, price: 100, leverage: 10)), service: service)
        let selection = try #require(model.leverageSelection)
        let other = try #require(selection.options.first { $0 != selection.selected })

        model.updateAutoclose(takeProfit: "150", stopLoss: model.stopLoss)
        let untouchedStopLoss = model.stopLoss
        selection.selected = other
        model.onChangeLeverage()

        #expect(model.takeProfit == "150")
        #expect(model.stopLoss != untouchedStopLoss)
    }

    @Test
    func makeAutocloseData() {
        let model = AmountPerpetualViewModel.mock(action: .open(data: .mock(direction: .long, price: 100, leverage: 3)))
        model.updateAutoclose(takeProfit: "100", stopLoss: "50")

        let data = model.makeAutocloseData(size: 1000)

        #expect(data.direction == .long)
        #expect(data.takeProfit == "100")
        #expect(data.stopLoss == "50")
        #expect(data.size == 1000)
    }

    @Test
    func makeTransferData() async throws {
        let open = try await transferData(AmountPerpetualViewModel.mock(), value: 100)
        let increase = try await transferData(AmountPerpetualViewModel.mock(action: .increase(data: .mock(direction: .long, price: 100, leverage: 3))), value: 200)
        let reduce = try await transferData(AmountPerpetualViewModel.mock(action: .reduce(data: .mock(direction: .long, price: 100, leverage: 3), position: PerpetualPosition.mock(marginAmount: 0.001).toGem())), value: 300)

        #expect(perpetualType(open).map {
            if case .open = $0 {
                true
            } else {
                false
            }
        } == true)
        #expect(perpetualType(increase).map {
            if case .increase = $0 {
                true
            } else {
                false
            }
        } == true)
        #expect(perpetualType(reduce).map {
            if case .reduce = $0 {
                true
            } else {
                false
            }
        } == true)
        #expect(open.value == "100")
        #expect(increase.value == "200")
        #expect(reduce.value == "300")
    }

    private func perpetualType(_ data: GemTransferData) -> Gemstone.PerpetualType? {
        guard case let .perpetual(_, perpetualType) = data.inputType else { return nil }
        return perpetualType
    }

    private func amountTitle(_ model: AmountPerpetualViewModel) -> String {
        model.request.amountType().title().title
    }

    private func transferData(_ model: AmountPerpetualViewModel, value: BigInt) async throws -> GemTransferData {
        try await GemAmountService.mock().transferData(asset: Asset.mock().toGem(), request: model.request, value: value, useMaxAmount: false)
    }

    private func amountInput(_ model: AmountPerpetualViewModel, _ assetData: AssetData) -> GemAmountInput {
        model.request.input(asset: model.asset.toGem(), balance: GemAssetBalance(assetData.balance, assetId: model.asset.id, isActive: assetData.metadata.isActive))
    }
}
