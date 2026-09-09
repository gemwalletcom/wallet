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
    func retryTransferShowsLoadingWhileTheTransferIsInFlight() {
        let viewModel = SwapButtonViewModel.mock(session: GemSwapSession.mockReady().startTransfer()!)

        #expect(viewModel.type == ButtonType.primary(.loading()))
    }

    @Test
    func insufficientBalanceNamesTheAssetAndDisablesTheButton() {
        let asset = AssetData.mock(asset: .mock(symbol: "BTC"))
        let viewModel = SwapButtonViewModel.mock(session: .mockReady(), value: 2, availableBalance: 1, fromAsset: asset)

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

extension SwapButtonViewModel {
    static func mock(
        session: GemSwapSession = .mock(),
        value: BigInt = 1,
        availableBalance: BigInt = 2,
        fromAsset: AssetData? = .mock(),
    ) -> SwapButtonViewModel {
        SwapButtonViewModel(
            state: session.viewState(value: value, availableBalance: availableBalance),
            fromAsset: fromAsset,
            onAction: {},
        )
    }
}

extension GemSwapRequest {
    static let mock = GemSwapRequest(
        payAssetId: AssetId.mockEthereum().identifier,
        receiveAssetId: AssetId.mockEthereumUSDT().identifier,
        value: 1_000_000_000_000_000_000,
        slippageBps: nil,
    )
}

extension GemSwapSession {
    static func mock() -> GemSwapSession {
        GemSwapSession(quotePhase: .noInput, transferPhase: .idle)
    }

    static func mockLoading() -> GemSwapSession {
        mock().onRequestChanged(request: .mock)
    }

    static func mockReady() -> GemSwapSession {
        mockLoading().onQuoteResults(results: GemSwapQuotesResult(request: .mock, quotes: [.mock()], error: nil))
    }

    static func mockFailed(_ error: SwapperError) -> GemSwapSession {
        mockLoading().onQuoteResults(results: GemSwapQuotesResult(request: .mock, quotes: [], error: error))
    }
}
