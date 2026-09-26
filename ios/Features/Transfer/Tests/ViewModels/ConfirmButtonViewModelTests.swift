// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.GemConfirmButton
import Localization
@testable import Primitives
import Testing
@testable import Transfer

struct ConfirmButtonViewModelTests {
    @Test
    func confirmTitle() {
        let model = ConfirmButtonViewModel(button: GemConfirmButton(kind: .confirm, state: .enabled, icon: .none), onAction: {})
        #expect(model.title == Localized.Transfer.confirm)
        #expect(!model.type.isDisabled)
    }

    @Test
    func retryTitle() {
        let model = ConfirmButtonViewModel(button: GemConfirmButton(kind: .retry, state: .enabled, icon: .none), onAction: {})
        #expect(model.title == Localized.Common.tryAgain)
    }

    @Test
    func disabledState() {
        let model = ConfirmButtonViewModel(button: GemConfirmButton(kind: .confirm, state: .disabled, icon: .none), onAction: {})
        #expect(model.type.isDisabled)
    }

    @Test
    func theButtonDrawsTheIconCoreChose() {
        #expect(ConfirmButtonViewModel(button: GemConfirmButton(kind: .confirm, state: .enabled, icon: .biometrics), onAction: {}).icon != nil)
        #expect(ConfirmButtonViewModel(button: GemConfirmButton(kind: .confirm, state: .loading, icon: .none), onAction: {}).icon == nil)
    }
}
