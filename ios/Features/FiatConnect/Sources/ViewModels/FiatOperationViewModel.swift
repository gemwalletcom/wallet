// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Components
import protocol Gemstone.GemFiatQuoteServiceProtocol
import GemstonePrimitives
import Formatters
import Foundation
import Localization
import Primitives
import PrimitivesComponents
import SwiftUI
import Validators

@MainActor
@Observable
final class FiatOperationViewModel {
    private let service: any GemFiatQuoteServiceProtocol
    private let type: FiatQuoteType
    private let asset: Asset
    private let walletId: WalletId
    private let currencyFormatter: CurrencyFormatter

    var quotesState: StateViewType<FiatQuotes> = .loading
    var selectedQuote: FiatQuote? {
        didSet { updateValidators() }
    }

    var loadTask: Task<Void, Never>?
    var amount: String
    var loadingAmount: Double?
    var inputValidationModel: InputValidationViewModel
    var availableBalance: BigInt = 0

    init(
        service: any GemFiatQuoteServiceProtocol,
        type: FiatQuoteType,
        asset: Asset,
        walletId: WalletId,
        currencyFormatter: CurrencyFormatter,
    ) {
        self.service = service
        self.type = type
        self.asset = asset
        self.walletId = walletId
        self.currencyFormatter = currencyFormatter
        amount = String(service.defaultAmount(quoteType: type.json()))
        inputValidationModel = InputValidationViewModel(
            mode: .onDemand,
            validators: [],
        )
        inputValidationModel.text = amount
        updateValidators()
    }

    var cryptoAmountValue: String {
        guard let selectedQuoteViewModel else { return " " }
        return "≈ \(selectedQuoteViewModel.amountText)"
    }

    var rateValue: String {
        guard let selectedQuoteViewModel else { return "" }
        return "1 \(asset.symbol) ≈ \(selectedQuoteViewModel.rateText)"
    }

    var emptyTitle: String {
        guard inputValidationModel.text.isEmptyOrZero else { return Localized.Buy.noResults }
        return switch type {
        case .buy: Localized.Input.enterAmountTo(Localized.Wallet.buy)
        case .sell: Localized.Input.enterAmountTo(Localized.Wallet.sell)
        }
    }

    func load() {
        guard let amount = Double(inputValidationModel.text), amount > 0 else {
            quotesState = .noData
            return
        }

        if inputValidationModel.isInvalid {
            if case let .data(fiatQuotes) = quotesState, fiatQuotes.amount == amount {
                return
            }
            quotesState = .noData
            return
        }

        if shouldSkipFetch(for: amount) {
            return
        }

        loadTask?.cancel()
        loadingAmount = amount

        loadTask = Task {
            setLoadingState()
            selectedQuote = nil

            do {
                let quotes = try await service.quotes(walletId: walletId, type: type, asset: asset, amount: amount)
                try Task.checkCancellation()

                if quotes.isNotEmpty {
                    selectedQuote = quotes.first
                    quotesState = .data(FiatQuotes(amount: amount, quotes: quotes))
                } else {
                    quotesState = .noData
                }
            } catch {
                guard !Task.isCancelled, !error.isCancelled else { return }
                quotesState = .error(error)
                debugLog("FiatOperationViewModel get quotes error: \(error)")
            }

            loadingAmount = nil
        }
    }

    func shouldSkipFetch(for amount: Double) -> Bool {
        loadingAmount == amount
    }

    func updateValidators() {
        let validator = FiatAmountValidator(
            service: service,
            type: type,
            asset: asset,
            quote: selectedQuote,
            availableBalance: availableBalance,
            currencyFormatter: currencyFormatter,
        )
        inputValidationModel.update(validators: [.assetAmount(decimals: 0, validators: [validator])])
    }

    func onAssetDataChange(_ assetData: AssetData) {
        guard availableBalance != assetData.balance.available else { return }
        availableBalance = assetData.balance.available
        updateValidators()
    }

    func setAmount(_ text: String) {
        if text != amount {
            selectedQuote = nil
            setLoadingState()
        }
        amount = text
        inputValidationModel.update(text: text)
        updateValidators()
    }

    func onChangeAmountText(_: String, text: String) {
        setAmount(text)
    }
}

extension FiatOperationViewModel {
    private var selectedQuoteViewModel: FiatQuoteViewModel? {
        guard let selectedQuote else { return nil }
        return FiatQuoteViewModel(asset: asset, quote: selectedQuote, formatter: currencyFormatter)
    }

    private func setLoadingState() {
        guard !quotesState.isLoading else { return }
        quotesState = .loading
    }
}
