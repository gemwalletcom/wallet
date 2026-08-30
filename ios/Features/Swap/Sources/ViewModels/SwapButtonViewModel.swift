// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import enum Gemstone.GemSwapButtonAction
import Localization
import Primitives
import Style
import SwiftUI

struct SwapButtonViewModel: StateButtonViewable {
    let buttonAction: GemSwapButtonAction

    private let swapState: SwapState
    private let fromAsset: AssetData?

    private let perform: @MainActor @Sendable () -> Void

    init(
        swapState: SwapState,
        buttonAction: GemSwapButtonAction,
        fromAsset: AssetData?,
        onAction: @MainActor @Sendable @escaping () -> Void,
    ) {
        self.swapState = swapState
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
        switch buttonAction {
        case .retryQuote: swapState.quotes.isLoading ? .primary(swapState.quotes) : .primary(.normal)
        case .insufficientBalance: .primary(.disabled)
        case .useMinimumAmount: .primary(.normal)
        case .retryTransfer: swapState.swapTransferData.isLoading ? .primary(swapState.swapTransferData) : .primary(.normal)
        case .swap: swapState.swapTransferData.isLoading ? .primary(swapState.swapTransferData) : .primary(swapState.quotes)
        }
    }

    var isVisible: Bool {
        !swapState.quotes.isNoData
    }

    func action() {
        perform()
    }
}
