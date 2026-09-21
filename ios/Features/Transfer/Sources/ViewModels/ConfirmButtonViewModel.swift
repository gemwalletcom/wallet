// Copyright (c). Gem Wallet. All rights reserved.

import Components
import struct Gemstone.GemConfirmButton
import enum Gemstone.GemKeystoreAuthentication
import GemstoneServices
import Primitives
import Style
import SwiftUI

struct ConfirmButtonViewModel: StateButtonViewable {
    private let onAction: @MainActor @Sendable () -> Void
    private let button: GemConfirmButton
    private let authentication: GemKeystoreAuthentication?

    init(
        button: GemConfirmButton,
        authentication: GemKeystoreAuthentication?,
        onAction: @MainActor @Sendable @escaping () -> Void,
    ) {
        self.button = button
        self.authentication = authentication
        self.onAction = onAction
    }

    var title: String {
        button.kind.title
    }

    var icon: Image? {
        guard button.kind == .confirm, button.state == .enabled,
              let authentication,
              let systemName = authentication.systemImage
        else { return nil }
        return Image(systemName: systemName)
    }

    var type: ButtonType {
        switch button.state {
        case .disabled: .primary(.disabled)
        case .loading: .primary(.loading())
        case .enabled: .primary(.normal)
        }
    }

    func action() {
        onAction()
    }
}

private extension GemKeystoreAuthentication {
    var systemImage: String? {
        switch self {
        case .biometrics: SystemImage.faceid
        case .passcode: SystemImage.lock
        case .none: .none
        }
    }
}
