// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import enum Gemstone.SwapperError
import struct Gemstone.SwapperQuote
import Localization
import Primitives
import PrimitivesComponents
import Style
import SwiftUI

enum SwapButtonAction: Equatable {
    case retryQuotes
    case retrySwap
    case insufficientBalance(asset: Asset)
    case useMinAmount(amount: String, asset: Asset)
    case swap
}

struct SwapButtonViewModel: StateButtonViewable {
    private let swapState: SwapState
    private let isAmountValid: Bool
    private let fromAsset: AssetData?

    private let perform: @MainActor @Sendable () -> Void

    init(
        swapState: SwapState,
        isAmountValid: Bool,
        fromAsset: AssetData?,
        onAction: @MainActor @Sendable @escaping () -> Void,
    ) {
        self.swapState = swapState
        self.isAmountValid = isAmountValid
        self.fromAsset = fromAsset
        perform = onAction
    }

    var title: String {
        switch buttonAction {
        case .retryQuotes, .retrySwap: Localized.Common.tryAgain
        case let .insufficientBalance(asset): Localized.Transfer.insufficientBalance(asset.symbol)
        case .useMinAmount: Localized.Swap.useMinimumAmount
        case .swap: Localized.Wallet.swap
        }
    }

    var buttonAction: SwapButtonAction {
        if let useMinAmount { return useMinAmount }
        if canRetryQuotes { return .retryQuotes }
        if canRetrySwap { return .retrySwap }
        if !isAmountValid, let fromAsset { return .insufficientBalance(asset: fromAsset.asset) }
        return .swap
    }

    var icon: Image? { nil }
    var type: ButtonType {
        switch buttonAction {
        case .retryQuotes: swapState.quotes.isLoading ? .primary(swapState.quotes) : .primary(.normal)
        case .insufficientBalance: .primary(.disabled)
        case .useMinAmount: .primary(.normal)
        case .retrySwap: swapState.swapTransferData.isLoading ? .primary(swapState.swapTransferData) : .primary(.normal)
        case .swap: swapState.swapTransferData.isLoading ? .primary(swapState.swapTransferData) : .primary(swapState.quotes)
        }
    }

    var isVisible: Bool { !swapState.quotes.isNoData }

    func action() { perform() }
}

// MARK: - Private

extension SwapButtonViewModel {
    private var useMinAmount: SwapButtonAction? {
        guard case let .error(error) = swapState.quotes,
              case let .InputAmountError(amount) = error as? SwapperError,
              let amount,
              let fromAsset else { return nil }
        return .useMinAmount(amount: amount, asset: fromAsset.asset)
    }

    private var canRetryQuotes: Bool {
        guard case let .error(error) = swapState.quotes,
              let retryableError = error as? RetryableError else { return false }
        return retryableError.isRetryAvailable
    }

    private var canRetrySwap: Bool {
        guard case let .error(error) = swapState.swapTransferData,
              let retryableError = error as? RetryableError else { return false }
        return retryableError.isRetryAvailable
    }
}
