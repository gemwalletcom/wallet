// Copyright (c). Gem Wallet. All rights reserved.

import Gemstone
import GemstonePrimitivesTestKit
import GemstoneServices
import GemstoneServicesTestKit
import Primitives
import PrimitivesComponents
import Testing
@testable import Settings

@MainActor
struct PreferencesViewModelTests {
    private func model(settings: GemSettingsServiceMock = GemSettingsServiceMock()) -> PreferencesViewModel {
        PreferencesViewModel(settings: settings, preferences: .mock())
    }

    @Test
    func theDefaultsComeFromCore() {
        let settings = GemSettingsServiceMock()
        settings.perpetualDefaults = GemPerpetualDefaults(leverage: 10, takeProfitPercent: 30, stopLossPercent: 15)
        let model = model(settings: settings)

        #expect(model.perpetualLeverage.value == 10)
        #expect(model.defaultLeverageValue == "10x")
        #expect(model.perpetualTakeProfit.value == 30)
        #expect(model.perpetualStopLoss.value == 15)
    }

    @Test
    func changingALeverageWritesEveryDefaultBack() {
        let settings = GemSettingsServiceMock()
        settings.perpetualDefaults = GemPerpetualDefaults(leverage: 3, takeProfitPercent: 25, stopLossPercent: 10)
        let model = model(settings: settings)

        model.perpetualLeverage = LeverageOption(value: 20)

        #expect(settings.storedDefaults.count == 1)
        #expect(settings.storedDefaults.last?.leverage == 20)
        #expect(settings.storedDefaults.last?.takeProfitPercent == 25)
        #expect(settings.storedDefaults.last?.stopLossPercent == 10)
    }

    @Test
    func changingTakeProfitAndStopLossWritesThemToo() {
        let settings = GemSettingsServiceMock()
        let model = model(settings: settings)

        model.perpetualTakeProfit = AutocloseOption(value: 50)
        model.perpetualStopLoss = AutocloseOption(value: 20)

        #expect(settings.storedDefaults.map(\.takeProfitPercent) == [50, 50])
        #expect(settings.storedDefaults.last?.stopLossPercent == 20)
    }

    @Test
    func aFailedWriteLeavesTheSelectionOnScreen() {
        let settings = GemSettingsServiceMock()
        settings.setDefaultsError = AnyError("preferences are read only")
        let model = model(settings: settings)

        model.perpetualLeverage = LeverageOption(value: 20)

        #expect(model.perpetualLeverage.value == 20)
        #expect(settings.storedDefaults.isEmpty)
    }

    @Test
    func theStateIsAskedForWithThePerpetualToggle() {
        let settings = GemSettingsServiceMock()
        let model = model(settings: settings)
        model.isPerpetualEnabled = false

        _ = model.state

        #expect(settings.perpetualsEnabledCalls.last == false)
    }

    @Test
    func eachPickerOpensOnItsOwn() {
        let model = model()

        model.onSelectLeverage()
        #expect(model.isPresentingLeveragePicker)
        #expect(model.isPresentingTakeProfitPicker == false)

        model.onSelectTakeProfit()
        #expect(model.isPresentingTakeProfitPicker)

        model.onSelectStopLoss()
        #expect(model.isPresentingStopLossPicker)
    }

    @Test
    func theOfferedOptionsAreNotEmpty() {
        let model = model()

        #expect(model.leverageOptions.isNotEmpty)
        #expect(model.takeProfitOptions.isNotEmpty)
        #expect(model.stopLossOptions.isNotEmpty)
    }
}
