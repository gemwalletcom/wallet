// Copyright (c). Gem Wallet. All rights reserved.

import Components
import enum Gemstone.GemSwapButtonAction
import Localization
import Primitives
import PrimitivesTestKit
import Style
@testable import Swap
import Testing

struct SwapButtonViewModelTests {
    @Test
    func retryTitleForBothRetryActions() {
        #expect(SwapButtonViewModel.mock(buttonAction: .retryQuote).title == Localized.Common.tryAgain)
        #expect(SwapButtonViewModel.mock(buttonAction: .retryTransfer).title == Localized.Common.tryAgain)
    }

    @Test
    func retryQuotesStaysNormalWhileQuotesAreIdle() {
        let viewModel = SwapButtonViewModel.mock(
            swapState: SwapState(quotes: .error(TestError())),
            buttonAction: .retryQuote,
        )

        #expect(viewModel.type == ButtonType.primary(.normal))
        #expect(viewModel.isVisible == true)
    }

    @Test
    func retryTransferShowsLoadingWhileTheTransferIsInFlight() {
        let viewModel = SwapButtonViewModel.mock(
            swapState: SwapState(quotes: .data([]), swapTransferData: .loading),
            buttonAction: .retryTransfer,
        )

        #expect(viewModel.type == ButtonType.primary(.loading()))
    }

    @Test
    func insufficientBalanceNamesTheAssetAndDisablesTheButton() {
        let asset = AssetData.mock(asset: .mock(symbol: "BTC"))
        let viewModel = SwapButtonViewModel.mock(
            swapState: SwapState(quotes: .data([])),
            buttonAction: .insufficientBalance,
            fromAsset: asset,
        )

        #expect(viewModel.title == Localized.Transfer.insufficientBalance("BTC"))
        #expect(viewModel.type == ButtonType.primary(.disabled))
    }

    @Test
    func useMinimumAmountStaysEnabled() {
        let viewModel = SwapButtonViewModel.mock(
            swapState: SwapState(quotes: .error(TestError())),
            buttonAction: .useMinimumAmount(amount: "100"),
        )

        #expect(viewModel.title == Localized.Swap.useMinimumAmount)
        #expect(viewModel.type == ButtonType.primary(.normal))
    }

    @Test
    func swapFollowsTheQuoteState() {
        #expect(SwapButtonViewModel.mock(swapState: SwapState(quotes: .data([]))).title == Localized.Wallet.swap)
        #expect(SwapButtonViewModel.mock(swapState: SwapState(quotes: .data([]))).type == ButtonType.primary(.normal))
        #expect(SwapButtonViewModel.mock(swapState: SwapState(quotes: .loading)).type == ButtonType.primary(.loading()))
        #expect(SwapButtonViewModel.mock(swapState: SwapState(quotes: .error(TestError()))).type == ButtonType.primary(.disabled))
    }

    @Test
    func hiddenWhenNoQuotes() {
        #expect(SwapButtonViewModel.mock(swapState: SwapState(quotes: .noData)).isVisible == false)
    }
}

extension SwapButtonViewModel {
    static func mock(
        swapState: SwapState = SwapState(quotes: .noData),
        buttonAction: GemSwapButtonAction = .swap,
        fromAsset: AssetData? = .mock(),
    ) -> SwapButtonViewModel {
        SwapButtonViewModel(
            swapState: swapState,
            buttonAction: buttonAction,
            fromAsset: fromAsset,
            onAction: {},
        )
    }
}

private struct TestError: Error {}
