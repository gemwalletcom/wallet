// Copyright (c). Gem Wallet. All rights reserved.

import Gemstone
import GemstonePrimitivesTestKit
import GemstoneServices
import GemstoneServicesTestKit
import Primitives
import PrimitivesComponents
@testable import Settings
import SettingsTestKit
import Testing

@MainActor
struct PreferencesViewModelTests {
    @Test
    func theDefaultsComeFromCore() {
        let settings = GemSettingsServiceMock()
        settings.perpetualDefaultsValue = GemPerpetualDefaults(leverage: 10, takeProfitPercent: 25, stopLossPercent: 5)
        let model = PreferencesViewModel.mock(settings: settings)

        _ = model.sections

        #expect(model.perpetualLeverage.value == 10)
        #expect(model.perpetualTakeProfit.value == 25)
        #expect(model.perpetualStopLoss.value == 5)
        #expect(settings.preferencesInputs.last?.perpetualDefaults == settings.perpetualDefaultsValue)
        #expect(model.perpetualLeverage.displayText == "10x")
        #expect(model.perpetualTakeProfit.displayText == "25%")
    }

    @Test
    func changingALeverageWritesEveryDefaultBack() {
        let settings = GemSettingsServiceMock()
        settings.perpetualDefaultsValue = GemPerpetualDefaults(leverage: 3, takeProfitPercent: 25, stopLossPercent: 10)
        let model = PreferencesViewModel.mock(settings: settings)

        model.perpetualLeverage = GemPickerOption(value: 20, label: .number(number: .mock(value: 20)))

        #expect(settings.storedDefaults.count == 1)
        #expect(settings.storedDefaults.last?.leverage == 20)
        #expect(settings.storedDefaults.last?.takeProfitPercent == 25)
        #expect(settings.storedDefaults.last?.stopLossPercent == 10)
    }

    @Test
    func changingTakeProfitAndStopLossWritesThemToo() {
        let settings = GemSettingsServiceMock()
        let model = PreferencesViewModel.mock(settings: settings)

        model.perpetualTakeProfit = GemPickerOption(value: 50, label: .number(number: .mock(value: 50)))
        model.perpetualStopLoss = GemPickerOption(value: 20, label: .number(number: .mock(value: 20)))

        #expect(settings.storedDefaults.map(\.takeProfitPercent) == [50, 50])
        #expect(settings.storedDefaults.last?.stopLossPercent == 20)
    }

    @Test
    func aFailedWriteLeavesTheSelectionOnScreen() {
        let settings = GemSettingsServiceMock()
        settings.setDefaultsError = AnyError("preferences are read only")
        let model = PreferencesViewModel.mock(settings: settings)

        model.perpetualLeverage = GemPickerOption(value: 20, label: .number(number: .mock(value: 20)))

        #expect(model.perpetualLeverage.value == 20)
        #expect(settings.storedDefaults.isEmpty)
    }

    @Test
    func theSectionsAreAskedForWithThePerpetualToggle() {
        let settings = GemSettingsServiceMock()
        let model = PreferencesViewModel.mock(settings: settings)
        model.onToggle(.perpetuals, false)

        _ = model.sections

        #expect(settings.preferencesInputs.last?.perpetualsEnabled == false)

        model.onToggle(.perpetuals, true)
        _ = model.sections

        #expect(settings.preferencesInputs.last?.perpetualsEnabled == true)
    }

    @Test
    func eachPickerOpensOnItsOwn() {
        let model = PreferencesViewModel.mock()

        model.onSelect(.perpetualLeverage)
        #expect(model.isPresentingLeveragePicker)
        #expect(model.isPresentingTakeProfitPicker == false)

        model.onSelect(.perpetualTakeProfit)
        #expect(model.isPresentingTakeProfitPicker)

        model.onSelect(.perpetualStopLoss)
        #expect(model.isPresentingStopLossPicker)
    }

    @Test
    func theOfferedOptionsAreNotEmpty() {
        let model = PreferencesViewModel.mock()

        #expect(model.leverageOptions.isNotEmpty)
        #expect(model.takeProfitOptions.isNotEmpty)
        #expect(model.stopLossOptions.isNotEmpty)
    }
}
