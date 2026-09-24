// Copyright (c). Gem Wallet. All rights reserved.

import class Gemstone.GemAmountService
import struct Gemstone.GemPerpetualAutoclose
import GemstonePrimitivesTestKit
import Primitives
import PrimitivesTestKit
import Testing
@testable import Transfer
import TransferTestKit

struct AmountPerpetualViewModelTests {
    @Test
    func title() {
        let openLong = AmountPerpetualViewModel.mock(action: .open(data: .mock(direction: .long)))
        let openShort = AmountPerpetualViewModel.mock(action: .open(data: .mock(direction: .short)))

        #expect(openLong.title == "Long")
        #expect(openShort.title == "Short")
    }

    @Test
    func increaseReduceTitle() {
        let increase = AmountPerpetualViewModel.mock(action: .increase(data: .mock(direction: .long)))
        let reduce = AmountPerpetualViewModel.mock(action: .reduce(data: .mock(), position: PerpetualPosition.mock(marginAmount: 0.001).toGem()))

        #expect(increase.title.contains("Long"))
        #expect(reduce.title.contains("Long"))
    }

    @Test
    func leverageSelection() {
        let open = AmountPerpetualViewModel.mock(action: .open(data: .mock(leverage: 10)))
        let increase = AmountPerpetualViewModel.mock(action: .increase(data: .mock()))

        #expect(open.leverageSelection != nil)
        #expect(open.leverageSelection?.isEnabled == true)
        #expect(increase.leverageSelection == nil)
    }

    @Test
    func isAutocloseEnabled() {
        let open = AmountPerpetualViewModel.mock()
        let increase = AmountPerpetualViewModel.mock(action: .increase(data: .mock()))
        let reduce = AmountPerpetualViewModel.mock(action: .reduce(data: .mock(), position: PerpetualPosition.mock(marginAmount: 0.001).toGem()))

        #expect(open.isAutocloseEnabled == true)
        #expect(increase.isAutocloseEnabled == false)
        #expect(reduce.isAutocloseEnabled == false)
    }

    @Test
    func availableValue() {
        let assetData = AssetData.mock(balance: .mock(available: 5000))

        let open = AmountPerpetualViewModel.mock()
        let reduce = AmountPerpetualViewModel.mock(action: .reduce(data: .mock(), position: PerpetualPosition.mock(marginAmount: 0.001).toGem()))

        #expect(open.input(from: assetData).availableValue == 5000)
        #expect(reduce.input(from: assetData).availableValue == 1000)
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
        let model = AmountPerpetualViewModel.mock(action: .open(data: .mock(leverage: 10)), service: service)
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
        let model = AmountPerpetualViewModel.mock(action: .open(data: .mock(direction: .long)))
        model.updateAutoclose(takeProfit: "100", stopLoss: "50")

        let data = model.makeAutocloseData(size: 1000)

        #expect(data.direction == .long)
        #expect(data.takeProfit == "100")
        #expect(data.stopLoss == "50")
        #expect(data.size == 1000)
    }

    @Test
    func makeTransferData() {
        let open = AmountPerpetualViewModel.mock().makeTransferData(value: 100, useMaxAmount: false)
        let increase = AmountPerpetualViewModel.mock(action: .increase(data: .mock())).makeTransferData(value: 200, useMaxAmount: false)
        let reduce = AmountPerpetualViewModel.mock(action: .reduce(data: .mock(), position: PerpetualPosition.mock(marginAmount: 0.001).toGem())).makeTransferData(value: 300, useMaxAmount: false)

        #expect(open.transactionType().toPrimitives() == .perpetualOpenPosition)
        #expect(increase.transactionType().toPrimitives() == .perpetualOpenPosition)
        #expect(reduce.transactionType().toPrimitives() == .perpetualClosePosition)
        #expect(open.value == "100")
        #expect(increase.value == "200")
        #expect(reduce.value == "300")
    }
}
