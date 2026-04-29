// Copyright (c). Gem Wallet. All rights reserved.

import BalanceServiceTestKit
import BigInt
import ChainServiceTestKit
import protocol Gemstone.GemSwapperProtocol
import enum Gemstone.SwapperError
import Keystore
import KeystoreTestKit
import PreferencesTestKit
import PriceServiceTestKit
import Primitives
import PrimitivesTestKit
@testable import Store
import StoreTestKit
@testable import Swap
import SwapService
import SwapServiceTestKit
import Testing

@MainActor
struct SwapSceneViewModelTests {
    @Test
    func toValue() async {
        #expect(await model().toValue == "250,000.00")
        #expect(await model(toValueMock: "1000000").toValue == "1.00")
        #expect(await model(toValueMock: "10000").toValue == "0.01")
        #expect(await model(toValueMock: "12").toValue == "0.000012")
    }

    @Test
    func additionalInfoVisibility() {
        let model = SwapSceneViewModel.mock()

        model.swapState.quotes = .loading
        #expect(model.shouldShowAdditionalInfo == false)

        model.swapState.quotes = .data([.mock()])
        #expect(model.shouldShowAdditionalInfo)
    }

    @Test
    func buttonViewModelFlow() {
        let model = SwapSceneViewModel.mock()

        model.swapState.quotes = .data([])
        #expect(model.buttonViewModel.buttonAction == SwapButtonAction.swap)
        #expect(model.buttonViewModel.isVisible)

        model.swapState.quotes = .error(TestError())
        #expect(model.buttonViewModel.buttonAction == SwapButtonAction.retryQuotes)

        model.swapState.quotes = .error(SwapperError.InputAmountError(minAmount: "1000"))
        #expect(model.buttonViewModel.buttonAction == SwapButtonAction.useMinAmount(amount: "1000", asset: .mockEthereum()))

        model.swapState.quotes = .data([])
        model.swapState.swapTransferData = .error(TestError())
        #expect(model.buttonViewModel.buttonAction == SwapButtonAction.retrySwap)

        model.swapState.quotes = .error(TestError())
        model.swapState.swapTransferData = .error(TestError())
        #expect(model.buttonViewModel.buttonAction == SwapButtonAction.retrySwap)
    }

    @Test
    func loadingFlagsSeparateQuoteAndTransferDataStates() {
        let model = SwapSceneViewModel.mock()

        model.swapState.quotes = .loading
        #expect(model.isQuoteLoading)
        #expect(model.isTransferDataLoading == false)
        #expect(model.isQuoteInteractionEnabled)
        #expect(model.isReceiveFieldLoading)

        model.swapState.quotes = .data([.mock()])
        model.swapState.swapTransferData = .loading
        #expect(model.isQuoteLoading == false)
        #expect(model.isTransferDataLoading)
        #expect(model.isQuoteInteractionEnabled == false)
        #expect(model.isReceiveFieldLoading == false)
    }

    @Test
    func fetchDoesNotRunWhileTransferDataLoading() async {
        let model = await model()
        let previousToValue = model.toValue
        let previousQuote = model.selectedSwapQuote

        model.swapState.swapTransferData = .loading
        await model.fetch()

        #expect(model.swapState.quotes.isLoading == false)
        #expect(model.toValue == previousToValue)
        #expect(model.selectedSwapQuote == previousQuote)
    }

    @Test
    func quoteChangingActionsClearTransferStateAndDisableProviderSelection() async {
        let model = await model()

        model.swapState.quotes = .data([.mock(), .mock(toValue: "260000000000")])
        model.swapState.swapTransferData = .error(TestError())

        model.onFinishSwapProviderSelection(.mock())
        #expect(model.swapState.swapTransferData.isNoData)

        model.swapState.swapTransferData = .loading
        #expect(model.swapDetailsViewModel?.allowSelectProvider == false)
    }

    @Test
    func cancelledTaskDoesNotUpdateStateWithError() async throws {
        let swapper = GemSwapperMock(
            fetchQuoteDelay: .milliseconds(100),
            fetchQuoteError: SwapperError.NoQuoteAvailable,
        )
        let model = SwapSceneViewModel.mock(swapper: swapper)

        let task = Task {
            await model.fetch()
        }

        try await Task.sleep(for: .milliseconds(50))
        task.cancel()
        await task.value

        if case .error = model.swapState.quotes {
            Issue.record("State should not be .error when Task is cancelled")
        }
    }

    @Test
    func emptyInputDoesNotApplyLateQuote() async throws {
        let swapper = GemSwapperMock(
            quotes: [.mock()],
            fetchQuoteDelay: .milliseconds(100),
        )
        let model = SwapSceneViewModel.mock(swapper: swapper)

        let task = Task {
            await model.fetch()
        }

        try await Task.sleep(for: .milliseconds(50))
        model.amountInputModel.text = "0"
        model.onChangeFromValue("1", "0")

        await task.value

        #expect(model.swapState.quotes.isNoData)
        #expect(model.toValue.isEmpty)
        #expect(model.selectedSwapQuote == nil)
    }

    @Test
    func clearingInputResetsQuoteImmediately() async {
        let model = await model()

        #expect(model.toValue.isNotEmpty)
        #expect(model.selectedSwapQuote != nil)

        model.amountInputModel.text = .empty
        model.onChangeFromValue("1", .empty)

        #expect(model.swapState.quotes.isNoData)
        #expect(model.toValue.isEmpty)
        #expect(model.selectedSwapQuote == nil)
    }

    @Test
    func emptyInputDoesNotApplyLateError() async throws {
        let swapper = GemSwapperMock(
            fetchQuoteDelay: .milliseconds(100),
            fetchQuoteError: SwapperError.NoQuoteAvailable,
        )
        let model = SwapSceneViewModel.mock(swapper: swapper)

        let task = Task {
            await model.fetch()
        }

        try await Task.sleep(for: .milliseconds(50))
        model.amountInputModel.text = "0"
        model.onChangeFromValue("1", "0")

        await task.value

        #expect(model.swapState.quotes.isNoData)
        #expect(model.toValue.isEmpty)
        #expect(model.selectedSwapQuote == nil)
    }

    @Test
    func fetchTriggerIsImmediate() {
        let model = SwapSceneViewModel.mock()

        model.fetchTrigger = nil
        model.onChangeFromValue("1", "2")

        #expect(model.fetchTrigger?.isImmediate == false)

        model.fetchTrigger = nil
        model.onSelectPercent(50)

        #expect(model.fetchTrigger?.isImmediate == true)

        model.fetchTrigger = nil
        model.onChangeToAsset(old: .mock(asset: .mockEthereum()), new: .mock(asset: .mockEthereumUSDT()))

        #expect(model.fetchTrigger?.isImmediate == true)

        model.fetchTrigger = nil
        model.swapState.quotes = .error(SwapperError.NoQuoteAvailable)
        model.buttonViewModel.action()

        #expect(model.fetchTrigger?.isImmediate == true)

        model.fetchTrigger = nil
        model.swapState.quotes = .error(SwapperError.InputAmountError(minAmount: "1000000000000000000"))
        model.buttonViewModel.action()

        #expect(model.fetchTrigger?.isImmediate == true)
    }

    // MARK: - Private methods

    private func model(
        toValueMock: String = "250000000000",
    ) async -> SwapSceneViewModel {
        let swapper = GemSwapperMock(quotes: [.mock(toValue: toValueMock)])
        let model = SwapSceneViewModel.mock(swapper: swapper)
        await model.fetch()
        return model
    }
}

extension SwapSceneViewModel {
    static func mock(swapper: GemSwapperProtocol = GemSwapperMock()) -> SwapSceneViewModel {
        let model = SwapSceneViewModel(
            preferences: .mock(),
            input: .init(
                wallet: .mock(accounts: [.mock(chain: .ethereum)]),
                pairSelector: SwapPairSelectorViewModel(fromAssetId: .mockEthereum(), toAssetId: nil),
            ),
            balanceUpdater: .mock(),
            priceUpdater: .mock(),
            swapQuotesProvider: SwapQuotesProvider(swapService: .mock(swapper: swapper)),
            swapQuoteDataProvider: SwapQuoteDataProvider(keystore: LocalKeystore.mock(), swapService: .mock(swapper: swapper)),
        )
        model.fromAssetQuery.value = .mock(asset: .mockEthereum(), balance: .mock())
        model.toAssetQuery.value = .mock(asset: .mockEthereumUSDT())
        model.amountInputModel.text = "1"

        return model
    }
}

private struct TestError: Error, RetryableError {
    var isRetryAvailable: Bool = true
}
