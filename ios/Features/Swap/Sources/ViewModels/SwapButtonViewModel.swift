// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import enum Gemstone.GemSwapButtonAction
import struct Gemstone.GemSwapViewState
import Localization
import Primitives
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
        switch buttonAction {
        case .retryQuote, .retryTransfer: Localized.Common.tryAgain
        case .insufficientBalance: Localized.Transfer.insufficientBalance(fromAsset?.asset.symbol ?? .empty)
        case .useMinimumAmount: Localized.Swap.useMinimumAmount
        case .swap: Localized.Wallet.swap
        }
    }

    var icon: Image? {
        nil
    }

    var type: ButtonType {
        switch state.buttonState {
        case .disabled: .primary(.disabled)
        case .loading: .primary(.loading(showProgress: true))
        case .enabled: .primary(.normal)
        }
    }

    var isVisible: Bool {
        !state.isInputEmpty
    }

    func action() {
        perform()
    }
}
