// Copyright (c). Gem Wallet. All rights reserved.

import BalanceServiceTestKit
import BigInt
import ChainServiceTestKit
import protocol Gemstone.GemSwapperProtocol
import enum Gemstone.SwapperError
import struct Gemstone.SwapperQuote
import Keystore
import KeystoreTestKit
import Preferences
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
    func assetIdsIgnoresPairOrder() {
        let model = SwapSceneViewModel.mock()
        let assetIds = model.assetIds

        model.fromAssetQuery.value = .mock(asset: .mockEthereumUSDT())
        model.toAssetQuery.value = .mock(asset: .mockEthereum(), balance: .mock())

        #expect(model.assetIds == assetIds)
        #expect(model.assetIds == [AssetId.mockEthereum(), AssetId.mockEthereumUSDT()])
    }

    @Test
    func toValue() async {
        #expect(await model().toValue == "250,000")
        #expect(await model(toValueMock: "1000000").toValue == "1")
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
    func changingReceiveAssetPreservesInputAmount() async {
        let model = await model()
        let oldAsset = model.toAsset

        model.swapState.swapTransferData = .error(TestError())
        model.fetchTrigger = nil
        model.toAssetQuery.value = .mock(asset: .mockBNB())
        model.onChangeToAsset(old: oldAsset, new: model.toAsset)

        #expect(model.amountInputModel.text == "1")
        #expect(model.toValue.isEmpty)
        #expect(model.selectedSwapQuote == nil)
        #expect(model.swapState.swapTransferData.isNoData)
        #expect(model.fetchTrigger?.isImmediate == true)
    }

    @Test
    func changingPayAssetClearsInputAmount() async {
        let model = await model()
        let oldAsset = model.fromAsset

        model.fromAssetQuery.value = .mock(asset: .mockBNB(), balance: .mock())
        model.onChangeFromAsset(old: oldAsset, new: model.fromAsset)

        #expect(model.amountInputModel.text.isEmpty)
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

    @Test
    func refreshedQuotesKeepSelectedProvider() async {
        let swapper = GemSwapperMock(
            quotes: [
                .mock(toValue: "260000000000", provider: .uniswapV3),
                .mock(toValue: "250000000000", provider: .thorchain),
            ],
        )
        let model = SwapSceneViewModel.mock(swapper: swapper)

        model.onFinishSwapProviderSelection(.mock(toValue: "249000000000", provider: .thorchain))
        await model.fetch()

        #expect(model.selectedSwapQuote?.data.provider.id == .thorchain)
        #expect(model.selectedSwapQuote?.toValue == "250000000000")
    }

    @Test
    func providerSelectionAppliesWithoutRefetch() async {
        let swapper = GemSwapperMock(
            quotes: [
                .mock(toValue: "260000000000", provider: .uniswapV3),
                .mock(toValue: "250000000000", provider: .thorchain),
            ],
        )
        let model = SwapSceneViewModel.mock(swapper: swapper)
        await model.fetch()

        #expect(model.selectedSwapQuote?.data.provider.id == .uniswapV3)

        model.onFinishSwapProviderSelection(.mock(toValue: "250000000000", provider: .thorchain))

        #expect(model.selectedSwapQuote?.data.provider.id == .thorchain)
    }

    @Test
    func selectedQuoteSurvivesQuotesReload() async {
        let model = await model()

        model.swapState.quotes = .loading

        #expect(model.selectedSwapQuote != nil)
        #expect(model.swapDetailsViewModel != nil)
    }

    @Test
    func increasedAmountSelectsBestProviderWithoutManualSelection() async {
        let model = SwapSceneViewModel.mock(swapQuotesProvider: SwapQuotesProviderMock())

        await model.fetch()

        #expect(model.selectedSwapQuote?.data.provider.id == .thorchain)

        model.amountInputModel.text = "4"
        await model.fetch()

        #expect(model.selectedSwapQuote?.data.provider.id == .uniswapV3)
        #expect(model.selectedSwapQuote?.toValue == "260000000000")
    }

    @Test
    func refreshedQuotesFallBackWhenSelectedProviderDisappears() async {
        let swapper = GemSwapperMock(quotes: [.mock(toValue: "260000000000", provider: .uniswapV3)])
        let model = SwapSceneViewModel.mock(swapper: swapper)

        model.onFinishSwapProviderSelection(.mock(toValue: "249000000000", provider: .thorchain))
        await model.fetch()

        #expect(model.selectedSwapQuote?.data.provider.id == .uniswapV3)
    }

    @Test
    func changedPairDropsManualProviderSelection() async {
        let swapper = GemSwapperMock(
            quotes: [
                .mock(toValue: "260000000000", provider: .uniswapV3),
                .mock(toValue: "250000000000", provider: .thorchain),
            ],
        )
        let model = SwapSceneViewModel.mock(swapper: swapper)

        model.onFinishSwapProviderSelection(.mock(toValue: "249000000000", provider: .thorchain))
        model.onChangeToAsset(old: .mock(asset: .mockEthereum()), new: .mock(asset: .mockEthereumUSDT()))
        await model.fetch()

        #expect(model.selectedSwapQuote?.data.provider.id == .uniswapV3)
    }

    @Test
    func slippagePersistsAcrossSessions() {
        let preferences = Preferences.mock()
        let model = SwapSceneViewModel.mock(preferences: preferences)
        #expect(model.selectedSlippage == .auto)

        model.onSelectSlippage(.manual(bps: 150))

        #expect(preferences.swapSlippage == .manual(bps: 150))
        #expect(SwapSceneViewModel.mock(preferences: preferences).selectedSlippage == .manual(bps: 150))
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
    static func mock(
        swapper: GemSwapperProtocol = GemSwapperMock(),
        swapQuotesProvider: (any SwapQuotesProvidable)? = nil,
        preferences: Preferences = .mock(),
    ) -> SwapSceneViewModel {
        let model = SwapSceneViewModel(
            preferences: preferences,
            input: .init(
                wallet: .mock(accounts: [.mock(chain: .ethereum)]),
                pairSelector: SwapPairSelectorViewModel(fromAssetId: .mockEthereum(), toAssetId: nil),
            ),
            balanceUpdater: .mock(),
            priceUpdater: .mock(),
            swapQuotesProvider: swapQuotesProvider ?? SwapQuotesProvider(swapService: .mock(swapper: swapper)),
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

private struct SwapQuotesProviderMock: SwapQuotesProvidable {
    func supportedAssets(for _: Primitives.AssetId) -> ([Primitives.Chain], [Primitives.AssetId]) {
        ([], [])
    }

    func fetchQuotes(
        wallet _: Wallet,
        fromAsset _: Asset,
        toAsset _: Asset,
        amount: BigInt,
        useMaxAmount _: Bool,
        slippage _: SwapSlippage,
    ) async throws -> [SwapperQuote] {
        guard amount > BigInt(stringLiteral: "2000000000000000000") else {
            return [.mock(toValue: "250000000000", provider: .thorchain)]
        }
        return [
            .mock(toValue: "260000000000", provider: .uniswapV3),
            .mock(toValue: "250000000000", provider: .thorchain),
        ]
    }
}
