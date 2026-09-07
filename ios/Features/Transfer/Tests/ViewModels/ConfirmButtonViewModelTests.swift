// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.GemConfirmButton
import Localization
@testable import Primitives
import Testing
@testable import Transfer
import TransferTestKit

struct ConfirmButtonViewModelTests {
    @Test
    func confirmTitle() {
        let model = ConfirmButtonViewModel(button: GemConfirmButton(kind: .confirm, state: .enabled), authentication: nil, onAction: {})
        #expect(model.title == Localized.Transfer.confirm)
        #expect(!model.type.isDisabled)
    }

    @Test
    func retryTitle() {
        let model = ConfirmButtonViewModel(button: GemConfirmButton(kind: .retry, state: .enabled), authentication: nil, onAction: {})
        #expect(model.title == Localized.Common.tryAgain)
    }

    @Test
    func disabledState() {
        let model = ConfirmButtonViewModel(button: GemConfirmButton(kind: .confirm, state: .disabled), authentication: nil, onAction: {})
        #expect(model.type.isDisabled)
    }

    @Test
    func loadingStateHidesTheAuthenticationIcon() {
        let model = ConfirmButtonViewModel(button: GemConfirmButton(kind: .confirm, state: .loading), authentication: .biometrics, onAction: {})
        #expect(model.icon == nil)
        #expect(ConfirmButtonViewModel(button: GemConfirmButton(kind: .confirm, state: .enabled), authentication: .biometrics, onAction: {}).icon != nil)
    }
}
