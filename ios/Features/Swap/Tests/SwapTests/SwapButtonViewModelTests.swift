// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Components
import struct Gemstone.GemSwapQuotesResult
import struct Gemstone.GemSwapRequest
import struct Gemstone.GemSwapSession
import enum Gemstone.SwapperError
import GemstonePrimitivesTestKit
import Localization
import Primitives
import PrimitivesTestKit
import Style
@testable import Swap
@testable import SwapTestKit
import Testing

struct SwapButtonViewModelTests {
    @Test
    func retryTitleForBothRetryActions() {
        #expect(SwapButtonViewModel.mock(session: .mock(quotePhase: .failed(request: .mock(), error: .NoQuoteAvailable), input: .mock())).title == Localized.Common.tryAgain)
        #expect(SwapButtonViewModel.mock(session: .mock(quotes: .mock(quotes: [.mock()]), selectedQuote: .mock(), quotePhase: .ready, transferPhase: .failed(request: .mock(), provider: .uniswapV3, error: .NoQuoteAvailable), input: .mock()))
            .title == Localized.Common.tryAgain)
    }

    @Test
    func retryQuotesStaysNormalWhileQuotesAreIdle() {
        let viewModel = SwapButtonViewModel.mock(session: .mock(quotePhase: .failed(request: .mock(), error: .NoQuoteAvailable), input: .mock()))

        #expect(viewModel.type == ButtonType.primary(.normal))
        #expect(viewModel.isVisible == true)
    }

    @Test
    func retryTransferShowsLoadingWhileTheTransferIsInFlight() throws {
        let viewModel = try SwapButtonViewModel.mock(session: #require(GemSwapSession.mock(quotes: .mock(quotes: [.mock()]), selectedQuote: .mock(), quotePhase: .ready, input: .mock()).startTransfer()))

        #expect(viewModel.type == ButtonType.primary(.loading()))
    }

    @Test
    func insufficientBalanceNamesTheAssetAndDisablesTheButton() {
        let asset = AssetData.mock(asset: .mock(symbol: "BTC"))
        let viewModel = SwapButtonViewModel.mock(session: .mock(quotes: .mock(quotes: [.mock()]), selectedQuote: .mock(), quotePhase: .ready, input: .mock(request: .mock(value: 2))), availableBalance: 1, fromAsset: asset)

        #expect(viewModel.title == Localized.Transfer.insufficientBalance("BTC"))
        #expect(viewModel.type == ButtonType.primary(.disabled))
    }

    @Test
    func useMinimumAmountStaysEnabled() {
        let viewModel = SwapButtonViewModel.mock(session: .mock(quotePhase: .failed(request: .mock(), error: .InputAmountError(minAmount: "100")), input: .mock()), availableBalance: 1000)

        #expect(viewModel.title == Localized.Swap.useMinimumAmount)
        #expect(viewModel.type == ButtonType.primary(.normal))
    }

    @Test
    func swapFollowsTheQuoteState() {
        #expect(SwapButtonViewModel.mock(session: .mock(quotes: .mock(quotes: [.mock()]), selectedQuote: .mock(), quotePhase: .ready, input: .mock())).title == Localized.Wallet.swap)
        #expect(SwapButtonViewModel.mock(session: .mock(quotes: .mock(quotes: [.mock()]), selectedQuote: .mock(), quotePhase: .ready, input: .mock())).type == ButtonType.primary(.normal))
        #expect(SwapButtonViewModel.mock(session: .mock(quotePhase: .loading(request: .mock()), input: .mock())).type == ButtonType.primary(.loading()))
        #expect(SwapButtonViewModel.mock(session: .mock(quotePhase: .failed(request: .mock(), error: .NoAvailableProvider), input: .mock())).type == ButtonType.primary(.disabled))
    }

    @Test
    func hiddenWhenNoQuotes() {
        #expect(SwapButtonViewModel.mock(session: .mock()).isVisible == false)
    }
}
