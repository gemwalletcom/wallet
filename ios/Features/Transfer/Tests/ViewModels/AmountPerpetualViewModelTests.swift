// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Primitives
import PrimitivesTestKit
import GemstonePrimitivesTestKit
import Testing
@testable import Transfer

struct AmountPerpetualViewModelTests {
    @Test
    func title() {
        let openLong = AmountPerpetualViewModel(asset: .mock(), data: .mock(positionAction: .open(.mock(direction: .long))), service: GemAmountServiceMock())
        let openShort = AmountPerpetualViewModel(asset: .mock(), data: .mock(positionAction: .open(.mock(direction: .short))), service: GemAmountServiceMock())

        #expect(openLong.title == "Long")
        #expect(openShort.title == "Short")
    }

    @Test
    func increaseReduceTitle() {
        let increase = AmountPerpetualViewModel(asset: .mock(), data: .mock(positionAction: .increase(.mock(direction: .long))), service: GemAmountServiceMock())
        let reduce = AmountPerpetualViewModel(asset: .mock(), data: .mock(positionAction: .reduce(.mock(), available: 1000, positionDirection: .long)), service: GemAmountServiceMock())

        #expect(increase.title.contains("Long"))
        #expect(reduce.title.contains("Long"))
    }

    @Test
    func leverageSelection() {
        let open = AmountPerpetualViewModel(asset: .mock(), data: .mock(positionAction: .open(.mock(leverage: 10))), service: GemAmountServiceMock())
        let increase = AmountPerpetualViewModel(asset: .mock(), data: .mock(positionAction: .increase(.mock())), service: GemAmountServiceMock())

        #expect(open.leverageSelection != nil)
        #expect(open.leverageSelection?.isEnabled == true)
        #expect(increase.leverageSelection == nil)
    }

    @Test
    func isAutocloseEnabled() {
        let open = AmountPerpetualViewModel(asset: .mock(), data: .mock(positionAction: .open(.mock())), service: GemAmountServiceMock())
        let increase = AmountPerpetualViewModel(asset: .mock(), data: .mock(positionAction: .increase(.mock())), service: GemAmountServiceMock())
        let reduce = AmountPerpetualViewModel(asset: .mock(), data: .mock(positionAction: .reduce(.mock(), available: 1000, positionDirection: .long)), service: GemAmountServiceMock())

        #expect(open.isAutocloseEnabled == true)
        #expect(increase.isAutocloseEnabled == false)
        #expect(reduce.isAutocloseEnabled == false)
    }

    @Test
    func availableValue() {
        let assetData = AssetData.mock(balance: .mock(available: 5000))

        let open = AmountPerpetualViewModel(asset: .mock(), data: .mock(positionAction: .open(.mock())), service: GemAmountServiceMock())
        let reduce = AmountPerpetualViewModel(asset: .mock(), data: .mock(positionAction: .reduce(.mock(), available: 1000, positionDirection: .long)), service: GemAmountServiceMock())

        #expect(open.availableValue(from: assetData) == 5000)
        #expect(reduce.availableValue(from: assetData) == 1000)
    }

    @Test
    func reserveForFee() {
        let model = AmountPerpetualViewModel(asset: .mock(), data: .mock(), service: GemAmountServiceMock())
        #expect(model.reserveForFee == .zero)
        #expect(model.shouldReserveFee(from: .mock()) == false)
    }

    @Test
    func autocloseText() {
        let model = AmountPerpetualViewModel(asset: .mock(), data: .mock(), service: GemAmountServiceMock())

        #expect(model.autocloseText.subtitle == "-")
        #expect(model.autocloseText.subtitleExtra == nil)

        model.takeProfit = "100"
        #expect(model.autocloseText.subtitle.contains("TP"))

        model.stopLoss = "50"
        #expect(model.autocloseText.subtitleExtra != nil)
    }

    @Test
    func makeAutocloseData() {
        let model = AmountPerpetualViewModel(asset: .mock(), data: .mock(positionAction: .open(.mock(direction: .long))), service: GemAmountServiceMock())
        model.takeProfit = "100"
        model.stopLoss = "50"

        let data = model.makeAutocloseData(size: 1000)

        #expect(data.direction == .long)
        #expect(data.takeProfit == "100")
        #expect(data.stopLoss == "50")
        #expect(data.size == 1000)
    }

    @Test
    func makeTransferData() throws {
        let open = try AmountPerpetualViewModel(asset: .mock(), data: .mock(positionAction: .open(.mock())), service: GemAmountServiceMock()).makeTransferData(value: 100, useMaxAmount: false)
        let increase = try AmountPerpetualViewModel(asset: .mock(), data: .mock(positionAction: .increase(.mock())), service: GemAmountServiceMock()).makeTransferData(value: 200, useMaxAmount: false)
        let reduce = try AmountPerpetualViewModel(asset: .mock(), data: .mock(positionAction: .reduce(.mock(), available: 1000, positionDirection: .long)), service: GemAmountServiceMock()).makeTransferData(value: 300, useMaxAmount: false)

        #expect(TransactionType(core: open.inputType.transactionType()) == .perpetualOpenPosition)
        #expect(TransactionType(core: increase.inputType.transactionType()) == .perpetualOpenPosition)
        #expect(TransactionType(core: reduce.inputType.transactionType()) == .perpetualClosePosition)
        #expect(open.value == "100")
        #expect(increase.value == "200")
        #expect(reduce.value == "300")
    }
}
