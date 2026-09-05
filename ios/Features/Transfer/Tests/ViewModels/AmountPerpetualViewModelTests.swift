// Copyright (c). Gem Wallet. All rights reserved.

import Primitives
import PrimitivesTestKit
import GemstonePrimitivesTestKit
import GemstoneServicesTestKit
import class Gemstone.GemAmountService
import Testing
@testable import Transfer

struct AmountPerpetualViewModelTests {
    @Test
    func title() {
        let openLong = AmountPerpetualViewModel(asset: .mock(), action: .open(data: .mock(direction: .long)), service: GemAmountServiceMock(builder: GemAmountService.mock()))
        let openShort = AmountPerpetualViewModel(asset: .mock(), action: .open(data: .mock(direction: .short)), service: GemAmountServiceMock(builder: GemAmountService.mock()))

        #expect(openLong.title == "Long")
        #expect(openShort.title == "Short")
    }

    @Test
    func increaseReduceTitle() {
        let increase = AmountPerpetualViewModel(asset: .mock(), action: .increase(data: .mock(direction: .long)), service: GemAmountServiceMock(builder: GemAmountService.mock()))
        let reduce = AmountPerpetualViewModel(asset: .mock(), action: .reduce(data: .mock(), available: 1000), service: GemAmountServiceMock(builder: GemAmountService.mock()))

        #expect(increase.title.contains("Long"))
        #expect(reduce.title.contains("Long"))
    }

    @Test
    func leverageSelection() {
        let open = AmountPerpetualViewModel(asset: .mock(), action: .open(data: .mock(leverage: 10)), service: GemAmountServiceMock(builder: GemAmountService.mock()))
        let increase = AmountPerpetualViewModel(asset: .mock(), action: .increase(data: .mock()), service: GemAmountServiceMock(builder: GemAmountService.mock()))

        #expect(open.leverageSelection != nil)
        #expect(open.leverageSelection?.isEnabled == true)
        #expect(increase.leverageSelection == nil)
    }

    @Test
    func isAutocloseEnabled() {
        let open = AmountPerpetualViewModel(asset: .mock(), action: .open(data: .mock()), service: GemAmountServiceMock(builder: GemAmountService.mock()))
        let increase = AmountPerpetualViewModel(asset: .mock(), action: .increase(data: .mock()), service: GemAmountServiceMock(builder: GemAmountService.mock()))
        let reduce = AmountPerpetualViewModel(asset: .mock(), action: .reduce(data: .mock(), available: 1000), service: GemAmountServiceMock(builder: GemAmountService.mock()))

        #expect(open.isAutocloseEnabled == true)
        #expect(increase.isAutocloseEnabled == false)
        #expect(reduce.isAutocloseEnabled == false)
    }

    @Test
    func availableValue() {
        let assetData = AssetData.mock(balance: .mock(available: 5000))

        let open = AmountPerpetualViewModel(asset: .mock(), action: .open(data: .mock()), service: GemAmountServiceMock(builder: GemAmountService.mock()))
        let reduce = AmountPerpetualViewModel(asset: .mock(), action: .reduce(data: .mock(), available: 1000), service: GemAmountServiceMock(builder: GemAmountService.mock()))

        #expect(open.input(from: assetData).availableValue == 5000)
        #expect(reduce.input(from: assetData).availableValue == 1000)
    }

    @Test
    func autocloseText() {
        let model = AmountPerpetualViewModel(asset: .mock(), action: .open(data: .mock()), service: GemAmountServiceMock(builder: GemAmountService.mock()))

        #expect(model.autocloseText.subtitle == "-")
        #expect(model.autocloseText.subtitleExtra == nil)

        model.takeProfit = "100"
        #expect(model.autocloseText.subtitle.contains("TP"))

        model.stopLoss = "50"
        #expect(model.autocloseText.subtitleExtra != nil)
    }

    @Test
    func makeAutocloseData() {
        let model = AmountPerpetualViewModel(asset: .mock(), action: .open(data: .mock(direction: .long)), service: GemAmountServiceMock(builder: GemAmountService.mock()))
        model.takeProfit = "100"
        model.stopLoss = "50"

        let data = model.makeAutocloseData(size: 1000)

        #expect(data.direction == .long)
        #expect(data.takeProfit == "100")
        #expect(data.stopLoss == "50")
        #expect(data.size == 1000)
    }

    @Test
    func makeTransferData() {
        let open = AmountPerpetualViewModel(asset: .mock(), action: .open(data: .mock()), service: GemAmountServiceMock(builder: GemAmountService.mock())).makeTransferData(value: 100, useMaxAmount: false)
        let increase = AmountPerpetualViewModel(asset: .mock(), action: .increase(data: .mock()), service: GemAmountServiceMock(builder: GemAmountService.mock())).makeTransferData(value: 200, useMaxAmount: false)
        let reduce = AmountPerpetualViewModel(asset: .mock(), action: .reduce(data: .mock(), available: 1000), service: GemAmountServiceMock(builder: GemAmountService.mock())).makeTransferData(value: 300, useMaxAmount: false)

        #expect(open.inputType.transactionType().map() == .perpetualOpenPosition)
        #expect(increase.inputType.transactionType().map() == .perpetualOpenPosition)
        #expect(reduce.inputType.transactionType().map() == .perpetualClosePosition)
        #expect(open.value == "100")
        #expect(increase.value == "200")
        #expect(reduce.value == "300")
    }
}
