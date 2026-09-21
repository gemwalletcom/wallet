// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Components
import struct Gemstone.GemSwapQuotesResult
import struct Gemstone.GemSwapRequest
import struct Gemstone.GemSwapSession
import enum Gemstone.SwapperError
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
        #expect(SwapButtonViewModel.mock(session: .mockFailed(.NoQuoteAvailable)).title == Localized.Common.tryAgain)
        #expect(SwapButtonViewModel.mock(session: .mockReady().failedTransfer(.NoQuoteAvailable)).title == Localized.Common.tryAgain)
    }

    @Test
    func retryQuotesStaysNormalWhileQuotesAreIdle() {
        let viewModel = SwapButtonViewModel.mock(session: .mockFailed(.NoQuoteAvailable))

        #expect(viewModel.type == ButtonType.primary(.normal))
        #expect(viewModel.isVisible == true)
    }

    @Test
    func retryTransferShowsLoadingWhileTheTransferIsInFlight() throws {
        let viewModel = try SwapButtonViewModel.mock(session: #require(GemSwapSession.mockReady().startTransfer()))

        #expect(viewModel.type == ButtonType.primary(.loading()))
    }

    @Test
    func insufficientBalanceNamesTheAssetAndDisablesTheButton() {
        let asset = AssetData.mock(asset: .mock(symbol: "BTC"))
        let viewModel = SwapButtonViewModel.mock(session: .mockReady(), availableBalance: 1, fromAsset: asset)

        #expect(viewModel.title == Localized.Transfer.insufficientBalance("BTC"))
        #expect(viewModel.type == ButtonType.primary(.disabled))
    }

    @Test
    func useMinimumAmountStaysEnabled() {
        let viewModel = SwapButtonViewModel.mock(session: .mockFailed(.InputAmountError(minAmount: "100")), availableBalance: 1000)

        #expect(viewModel.title == Localized.Swap.useMinimumAmount)
        #expect(viewModel.type == ButtonType.primary(.normal))
    }

    @Test
    func swapFollowsTheQuoteState() {
        #expect(SwapButtonViewModel.mock(session: .mockReady()).title == Localized.Wallet.swap)
        #expect(SwapButtonViewModel.mock(session: .mockReady()).type == ButtonType.primary(.normal))
        #expect(SwapButtonViewModel.mock(session: .mockLoading()).type == ButtonType.primary(.loading()))
        #expect(SwapButtonViewModel.mock(session: .mockFailed(.NoAvailableProvider)).type == ButtonType.primary(.disabled))
    }

    @Test
    func hiddenWhenNoQuotes() {
        #expect(SwapButtonViewModel.mock(session: .mock()).isVisible == false)
    }
}
