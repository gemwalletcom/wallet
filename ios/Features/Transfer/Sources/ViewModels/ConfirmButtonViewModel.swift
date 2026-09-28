// Copyright (c). Gem Wallet. All rights reserved.

import Components
import struct Gemstone.GemConfirmButton
import Primitives
import PrimitivesComponents
import Style
import SwiftUI

struct ConfirmButtonViewModel: StateButtonViewable {
    private let onAction: @MainActor @Sendable () -> Void
    private let button: GemConfirmButton

    init(
        button: GemConfirmButton,
        onAction: @MainActor @Sendable @escaping () -> Void,
    ) {
        self.button = button
        self.onAction = onAction
    }

    var title: String {
        button.kind.title
    }

    var icon: Image? {
        button.icon.image
    }

    var type: ButtonType {
        .primary(button.state.state)
    }

    func action() {
        onAction()
    }
}
