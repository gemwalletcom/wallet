// Copyright (c). Gem Wallet. All rights reserved.

import Localization
@testable import Primitives
import Testing
@testable import Transfer
import TransferTestKit

struct ConfirmButtonViewModelTests {
    @Test
    func loaded() {
        let model = ConfirmButtonViewModel(state: .data(.mock()), authentication: nil, onAction: {})
        #expect(model.title == Localized.Transfer.confirm)
    }

    @Test
    func error() {
        let model = ConfirmButtonViewModel(state: .error(AnyError("test")), authentication: nil, onAction: {})
        #expect(model.title == Localized.Common.tryAgain)
    }

    @Test
    func disabledWhenForceDisabled() {
        let model = ConfirmButtonViewModel(state: .data(.mock()), authentication: nil, isDisabled: true, onAction: {})
        #expect(model.type.isDisabled)
    }

    @Test
    func enabledWhenNotForceDisabled() {
        let model = ConfirmButtonViewModel(state: .data(.mock()), authentication: nil, isDisabled: false, onAction: {})
        #expect(!model.type.isDisabled)
    }
}
