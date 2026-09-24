// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import enum Gemstone.GemSwapButtonAction
import struct Gemstone.GemSwapViewState
import Localization
import Primitives
import PrimitivesComponents
import Style
import SwiftUI

struct SwapButtonViewModel: StateButtonViewable {
    private let state: GemSwapViewState
    private let fromAsset: AssetData?

    private let perform: @MainActor @Sendable () -> Void

    init(
        state: GemSwapViewState,
        fromAsset: AssetData?,
        onAction: @MainActor @Sendable @escaping () -> Void,
    ) {
        self.state = state
        self.fromAsset = fromAsset
        perform = onAction
    }

    var buttonAction: GemSwapButtonAction {
        state.buttonAction
    }

    var title: String {
        buttonAction.title(symbol: fromAsset?.asset.symbol ?? .empty)
    }

    var icon: Image? {
        nil
    }

    var type: ButtonType {
        .primary(state.buttonState.state)
    }

    var isVisible: Bool {
        !state.isInputEmpty
    }

    func action() {
        perform()
    }
}
