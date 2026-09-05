// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import enum Gemstone.GemSwapButtonAction
import struct Gemstone.GemSwapSession
import Localization
import Primitives
import Style
import SwiftUI

struct SwapButtonViewModel: StateButtonViewable {
    let buttonAction: GemSwapButtonAction

    private let session: GemSwapSession
    private let fromAsset: AssetData?

    private let perform: @MainActor @Sendable () -> Void

    init(
        session: GemSwapSession,
        buttonAction: GemSwapButtonAction,
        fromAsset: AssetData?,
        onAction: @MainActor @Sendable @escaping () -> Void,
    ) {
        self.session = session
        self.buttonAction = buttonAction
        self.fromAsset = fromAsset
        perform = onAction
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
        switch session.buttonState(action: buttonAction) {
        case .disabled: .primary(.disabled)
        case .loading: .primary(.loading(showProgress: true))
        case .enabled: .primary(.normal)
        }
    }

    var isVisible: Bool {
        !session.isInputEmpty()
    }

    func action() {
        perform()
    }
}
