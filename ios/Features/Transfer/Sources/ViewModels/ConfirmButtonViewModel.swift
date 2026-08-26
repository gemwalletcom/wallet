// Copyright (c). Gem Wallet. All rights reserved.

import Components
import GemstoneServices
import Localization
import Primitives
import Style
import SwiftUI

struct ConfirmButtonViewModel: StateButtonViewable {
    private let onAction: @MainActor @Sendable () -> Void
    private let state: StateViewType<ConfirmTransferInput>
    private let authentication: KeystoreAuthentication?
    private let isDisabled: Bool

    init(
        state: StateViewType<ConfirmTransferInput>,
        authentication: KeystoreAuthentication?,
        isDisabled: Bool = false,
        onAction: @MainActor @Sendable @escaping () -> Void,
    ) {
        self.state = state
        self.authentication = authentication
        self.isDisabled = isDisabled
        self.onAction = onAction
    }

    var title: String {
        state.isError ? Localized.Common.tryAgain : Localized.Transfer.confirm
    }

    var icon: Image? {
        guard !state.isError, state.value?.transferAmount.isSuccess == true,
              let authentication,
              let systemName = KeystoreAuthenticationViewModel(authentication: authentication).authenticationImage
        else { return nil }
        return Image(systemName: systemName)
    }

    var type: ButtonType {
        let isDisabled = isDisabled || state.value?.transferAmount.isFailure == true
        return .primary(state, isDisabled: isDisabled)
    }

    func action() {
        onAction()
    }
}
