// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import PaymentService
import Primitives
import SigningRequestService
import TransactionStateService

public final class PaymentManager: Sendable {
    private let service: any PaymentServiceable
    private let executor: PaymentActionExecutor
    private let presenter: any PaymentSheetPresentable
    private let assetsProvider: any PaymentAssetsProvidable
    private let transactionStateScheduler: TransactionStateScheduler
    private let queue = PaymentQueue()

    public init(
        service: any PaymentServiceable,
        executor: PaymentActionExecutor,
        presenter: any PaymentSheetPresentable,
        assetsProvider: any PaymentAssetsProvidable,
        transactionStateScheduler: TransactionStateScheduler,
    ) {
        self.service = service
        self.executor = executor
        self.presenter = presenter
        self.assetsProvider = assetsProvider
        self.transactionStateScheduler = transactionStateScheduler
    }

    public func pay(link: PaymentLink, wallet: Wallet) async throws -> PaymentOutcome {
        try await queue.enqueue(paymentId: link.id) { [self] in
            try await perform(link: link, wallet: wallet)
        }.value
    }
}

// MARK: - Private

extension PaymentManager {
    @MainActor
    private func perform(link: PaymentLink, wallet: Wallet) async throws -> PaymentOutcome {
        do {
            let quotes: PaymentQuotes
            switch try await service.getPaymentOptions(link: link, wallet: wallet) {
            case let .outcome(outcome):
                return outcome
            case let .quotes(value):
                quotes = value
            }
            let quote = try await select(quotes: quotes, wallet: wallet)
            return try await submit(provider: link.provider, quotes: quotes, quote: quote, wallet: wallet)
        } catch SigningRequestError.userCancelled {
            return PaymentOutcome(status: .cancelled, transactionId: .none)
        }
    }

    @MainActor
    private func select(quotes: PaymentQuotes, wallet: Wallet) async throws -> PaymentQuote {
        guard let first = quotes.quotes.first else {
            throw PaymentLinkError.noQuotes
        }
        guard quotes.quotes.count > 1 else {
            return first
        }
        let assetsData = assetsProvider.assetsData(walletId: wallet.id, assetIds: quotes.quotes.map(\.amount.assetId))
        let selected = try await presenter.selectPaymentQuote(
            request: PaymentQuotesRequest(
                id: first.paymentId,
                quotes: quotes,
                wallet: wallet,
                assetsData: assetsData,
            ),
        )
        guard let quote = quotes.quotes.first(where: { $0.id == selected }) else {
            throw PaymentLinkError.quoteUnavailable
        }
        return quote
    }

    @MainActor
    private func submit(provider: PaymentProviderName, quotes: PaymentQuotes, quote: PaymentQuote, wallet: Wallet) async throws -> PaymentOutcome {
        if let url = quote.collectDataUrl {
            try await collectData(paymentId: quote.paymentId, url: url)
        }
        let payment = try await service.getPreparedPayment(provider: provider, quotes: quotes, quote: quote, wallet: wallet)
        let isRelayed = payment.actions.allSatisfy { action in
            switch action {
            case .signMessage, .signTransaction, .approveToken: true
            case .sendTransaction: false
            }
        }
        let results = try await executor.perform(
            actions: payment.actions,
            paymentId: payment.quote.paymentId,
            appMetadata: TransactionAppMetadata(merchant: payment.quotes.merchant),
            payment: PaymentData(provider: provider, quotes: payment.quotes, quote: payment.quote),
            wallet: wallet,
            onSubmitted: { [self] in
                guard isRelayed else {
                    return
                }
                save(provider: provider, payment: payment, wallet: wallet)
            },
        )
        do {
            return try await service.confirmPayment(provider: provider, quote: payment.quote, actionResults: results.results)
        } catch {
            debugLog("confirm payment error: \(error)")
            return PaymentOutcome(status: .processing, transactionId: results.transactionHash)
        }
    }

    @MainActor
    private func collectData(paymentId: String, url: String) async throws {
        guard let url = url.asURL else {
            throw PaymentLinkError.invalidDataCollectionUrl
        }
        _ = try await presenter.collectPaymentData(request: PaymentDataCollectionRequest(id: paymentId, url: url))
    }

    @MainActor
    private func save(provider: PaymentProviderName, payment: PreparedPayment, wallet: Wallet) {
        do {
            let transaction = try PaymentTransactionFactory.makePendingPayment(
                provider: provider,
                quote: payment.quote,
                merchant: payment.quotes.merchant,
                wallet: wallet,
            )
            try transactionStateScheduler.addTransactions(wallet: wallet, transactions: [transaction])
        } catch {
            debugLog("PaymentManager record payment error: \(error)")
        }
    }
}
