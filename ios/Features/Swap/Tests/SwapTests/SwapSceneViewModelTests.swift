// Copyright (c). Gem Wallet. All rights reserved.

import GemstonePrimitivesTestKit
import GemstoneServicesTestKit
import BigInt
import protocol Gemstone.GemSwapServiceProtocol
import enum Gemstone.GemSwapButtonAction
import class Gemstone.GemSwapQuoteService
import enum Gemstone.SwapperError
import struct Gemstone.GemSwapPairSuggestion
import struct Gemstone.SwapperQuote
import GemstoneServices
import Primitives
import PrimitivesTestKit
@testable import Store
import StoreTestKit
@testable import Swap
import class Gemstone.GemPreferencesService
import protocol Gemstone.GemPreferencesServiceProtocol
import GemstonePrimitives
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
    func suggestPairAppliesTheCoreSuggestion() async {
        let suggestion = GemSwapPairSuggestion(payAssetId: AssetId.mockEthereum().identifier, receiveAssetId: AssetId.mockEthereumUSDT().identifier)
        let model = SwapSceneViewModel.mock(swapService: GemSwapServiceMock(pairSuggestion: suggestion))

        await model.suggestPair()

        #expect(model.pairSelectorModel.fromAssetId == .mockEthereum())
        #expect(model.pairSelectorModel.toAssetId == .mockEthereumUSDT())
    }

    @Test
    func suggestPairKeepsAnAlreadySelectedReceiveAsset() async {
        let suggestion = GemSwapPairSuggestion(payAssetId: AssetId.mockEthereum().identifier, receiveAssetId: AssetId.mockSolana().identifier)
        let model = SwapSceneViewModel.mock(
            swapService: GemSwapServiceMock(pairSuggestion: suggestion),
            pairSelector: SwapPairSelectorViewModel(fromAssetId: .mockEthereum(), toAssetId: .mockEthereumUSDT()),
        )

        await model.suggestPair()

        #expect(model.pairSelectorModel.toAssetId == .mockEthereumUSDT())
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
        #expect(model.buttonViewModel.buttonAction == GemSwapButtonAction.swap)
        #expect(model.buttonViewModel.isVisible)

        model.swapState.quotes = .error(SwapperError.NoQuoteAvailable)
        #expect(model.buttonViewModel.buttonAction == GemSwapButtonAction.retryQuote)

        model.swapState.quotes = .error(SwapperError.InputAmountError(minAmount: "1000"))
        #expect(model.buttonViewModel.buttonAction == GemSwapButtonAction.useMinimumAmount(value: "1000"))

        model.swapState.quotes = .data([])
        model.swapState.swapTransferData = .error(SwapperError.NoQuoteAvailable)
        #expect(model.buttonViewModel.buttonAction == GemSwapButtonAction.retryTransfer)

        model.swapState.quotes = .error(SwapperError.NoQuoteAvailable)
        model.swapState.swapTransferData = .error(SwapperError.NoQuoteAvailable)
        #expect(model.buttonViewModel.buttonAction == GemSwapButtonAction.retryTransfer)
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
        await model.load()

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
        let swapService = GemSwapServiceMock(
            quotesDelay: .milliseconds(100),
            quotesError: SwapperError.NoQuoteAvailable,
        )
        let model = SwapSceneViewModel.mock(swapService: swapService)

        let task = Task {
            await model.load()
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
        let swapService = GemSwapServiceMock(
            quotes: [.mock()],
            quotesDelay: .milliseconds(100),
        )
        let model = SwapSceneViewModel.mock(swapService: swapService)

        let task = Task {
            await model.load()
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
        let swapService = GemSwapServiceMock(
            quotesDelay: .milliseconds(100),
            quotesError: SwapperError.NoQuoteAvailable,
        )
        let model = SwapSceneViewModel.mock(swapService: swapService)

        let task = Task {
            await model.load()
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
        model.loadTrigger = nil
        model.toAssetQuery.value = .mock(asset: .mockBNB())
        model.onChangeToAsset(old: oldAsset, new: model.toAsset)

        #expect(model.amountInputModel.text == "1")
        #expect(model.toValue.isEmpty)
        #expect(model.selectedSwapQuote == nil)
        #expect(model.swapState.swapTransferData.isNoData)
        #expect(model.loadTrigger?.isImmediate == true)
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
    func loadTriggerIsImmediate() {
        let model = SwapSceneViewModel.mock()

        model.loadTrigger = nil
        model.onChangeFromValue("1", "2")

        #expect(model.loadTrigger?.isImmediate == false)

        model.loadTrigger = nil
        model.onSelectPercent(50)

        #expect(model.loadTrigger?.isImmediate == true)

        model.loadTrigger = nil
        model.onChangeToAsset(old: .mock(asset: .mockEthereum()), new: .mock(asset: .mockEthereumUSDT()))

        #expect(model.loadTrigger?.isImmediate == true)

        model.loadTrigger = nil
        model.swapState.quotes = .error(SwapperError.NoQuoteAvailable)
        model.buttonViewModel.action()

        #expect(model.loadTrigger?.isImmediate == true)

        model.loadTrigger = nil
        model.swapState.quotes = .error(SwapperError.InputAmountError(minAmount: "1000000000000000000"))
        model.buttonViewModel.action()

        #expect(model.loadTrigger?.isImmediate == true)
    }

    @Test
    func retryQuoteUpdatesLoadTrigger() {
        let model = SwapSceneViewModel.mock()

        model.swapState.quotes = .error(SwapperError.NoQuoteAvailable)
        model.buttonViewModel.action()
        let firstRetry = model.loadTrigger
        model.buttonViewModel.action()

        #expect(model.loadTrigger != firstRetry)
    }

    @Test
    func refreshedQuotesKeepSelectedProvider() async {
        let swapService = GemSwapServiceMock(
            quotes: [
                .mock(toValue: "260000000000", provider: .uniswapV3),
                .mock(toValue: "250000000000", provider: .thorchain),
            ],
        )
        let model = SwapSceneViewModel.mock(swapService: swapService)

        model.onFinishSwapProviderSelection(.mock(toValue: "249000000000", provider: .thorchain))
        await model.load()

        #expect(model.selectedSwapQuote?.data.provider.id == .thorchain)
        #expect(model.selectedSwapQuote?.toValue == "250000000000")
    }

    @Test
    func providerSelectionAppliesWithoutRefetch() async {
        let swapService = GemSwapServiceMock(
            quotes: [
                .mock(toValue: "260000000000", provider: .uniswapV3),
                .mock(toValue: "250000000000", provider: .thorchain),
            ],
        )
        let model = SwapSceneViewModel.mock(swapService: swapService)
        await model.load()

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
        let model = SwapSceneViewModel.mock(swapService: GemSwapServiceMock(quotes: quotesByAmount))

        await model.load()

        #expect(model.selectedSwapQuote?.data.provider.id == .thorchain)

        model.amountInputModel.text = "4"
        await model.load()

        #expect(model.selectedSwapQuote?.data.provider.id == .uniswapV3)
        #expect(model.selectedSwapQuote?.toValue == "260000000000")
    }

    @Test
    func refreshedQuotesFallBackWhenSelectedProviderDisappears() async {
        let swapService = GemSwapServiceMock(quotes: [.mock(toValue: "260000000000", provider: .uniswapV3)])
        let model = SwapSceneViewModel.mock(swapService: swapService)

        model.onFinishSwapProviderSelection(.mock(toValue: "249000000000", provider: .thorchain))
        await model.load()

        #expect(model.selectedSwapQuote?.data.provider.id == .uniswapV3)
    }

    @Test
    func changedPairDropsManualProviderSelection() async {
        let swapService = GemSwapServiceMock(
            quotes: [
                .mock(toValue: "260000000000", provider: .uniswapV3),
                .mock(toValue: "250000000000", provider: .thorchain),
            ],
        )
        let model = SwapSceneViewModel.mock(swapService: swapService)

        model.onFinishSwapProviderSelection(.mock(toValue: "249000000000", provider: .thorchain))
        model.onChangeToAsset(old: .mock(asset: .mockEthereum()), new: .mock(asset: .mockEthereumUSDT()))
        await model.load()

        #expect(model.selectedSwapQuote?.data.provider.id == .uniswapV3)
    }

    @Test
    func slippagePersistsAcrossSessions() {
        let preferencesService = GemPreferencesService(store: GemPreferencesStoreMock())
        let model = SwapSceneViewModel.mock(preferencesService: preferencesService)
        #expect(model.selectedSlippage == .auto)

        model.onSelectSlippage(.manual(bps: 150))

        #expect(preferencesService.swapSlippage == .manual(bps: 150))
        #expect(SwapSceneViewModel.mock(preferencesService: preferencesService).selectedSlippage == .manual(bps: 150))
    }

    @Test
    func minimumAmountIsOfferedOnlyWhenTheBalanceCoversIt() async {
        let model = SwapSceneViewModel.mock()

        model.swapState.quotes = .error(SwapperError.InputAmountError(minAmount: "900000000000000000"))
        #expect(model.buttonViewModel.buttonAction == .useMinimumAmount(value: "900000000000000000"))

        model.swapState.quotes = .error(SwapperError.InputAmountError(minAmount: "2000000000000000000"))
        #expect(model.buttonViewModel.buttonAction == .insufficientBalance)
    }

    @Test
    func unaffordableAmountBlocksTheButtonBeforeAnyQuote() {
        let model = SwapSceneViewModel.mock()

        model.amountInputModel.text = "2"

        #expect(model.swapState.quotes.isNoData)
        #expect(model.buttonViewModel.buttonAction == .insufficientBalance)

        model.amountInputModel.text = "1"

        #expect(model.buttonViewModel.buttonAction == .swap)
    }

    @Test
    func onlyRetryableFailuresOfferARetry() {
        let model = SwapSceneViewModel.mock()

        model.swapState.quotes = .error(SwapperError.NoQuoteAvailable)
        #expect(model.buttonViewModel.buttonAction == .retryQuote)

        model.swapState.quotes = .error(SwapperError.NoAvailableProvider)
        #expect(model.buttonViewModel.buttonAction == .swap)

        model.swapState.swapTransferData = .error(SwapperError.TransactionError("nonce"))
        #expect(model.buttonViewModel.buttonAction == .retryTransfer)

        model.swapState.swapTransferData = .error(SwapperError.NotSupportedAsset)
        #expect(model.buttonViewModel.buttonAction == .swap)
    }

    // MARK: - Private methods

    private func model(
        toValueMock: String = "250000000000",
    ) async -> SwapSceneViewModel {
        let swapService = GemSwapServiceMock(quotes: [.mock(toValue: toValueMock)])
        let model = SwapSceneViewModel.mock(swapService: swapService)
        await model.load()
        return model
    }
}

extension SwapSceneViewModel {
    static func mock(
        swapService: any GemSwapServiceProtocol = GemSwapServiceMock(),
        preferencesService: any GemPreferencesServiceProtocol = GemPreferencesService(store: GemPreferencesStoreMock()),
        pairSelector: SwapPairSelectorViewModel = SwapPairSelectorViewModel(fromAssetId: .mockEthereum(), toAssetId: nil),
    ) -> SwapSceneViewModel {
        let model = SwapSceneViewModel(
            preferencesService: preferencesService,
            input: .init(
                wallet: .mock(accounts: [.mock(chain: .ethereum)]),
                pairSelector: pairSelector,
            ),
            balanceService: GemBalanceServiceMock(),
            priceUpdater: .mock(),
            swapService: swapService,
            swapQuoteService: GemSwapQuoteService(),
        )
        model.fromAssetQuery.value = .mock(asset: .mockEthereum(), balance: .mock())
        model.toAssetQuery.value = .mock(asset: .mockEthereumUSDT())
        model.amountInputModel.text = "1"

        return model
    }
}

private struct TestError: Error {}

private let quotesByAmount: @Sendable (BigInt) -> [SwapperQuote] = { amount in
    guard amount > BigInt(stringLiteral: "2000000000000000000") else {
        return [.mock(toValue: "250000000000", provider: .thorchain)]
    }
    return [
        .mock(toValue: "260000000000", provider: .uniswapV3),
        .mock(toValue: "250000000000", provider: .thorchain),
    ]
}
