// Copyright (c). Gem Wallet. All rights reserved.

import BalanceServiceTestKit
import Foundation
@testable import Payments
import PaymentService
import Primitives
import PrimitivesTestKit
import Testing

@MainActor
struct PaymentSceneViewModelTests {
    @Test
    func fetchQuotesSelectsTheFirst() {
        let model = model(quotes: .mock(quotes: [.mock(id: "eth"), .mock(id: "usdc")]))

        #expect(model.quoteItems.count == 2)
        #expect(model.state.selectedQuoteId == "eth")
        #expect(model.showsQuoteSelection)
        #expect(model.buttonModel.buttonAction == .confirm)
    }

    @Test
    func singleQuoteSkipsSelection() {
        let model = model(quotes: .mock(quotes: [.mock(id: "eth")]))

        #expect(!model.showsQuoteSelection)
        #expect(model.state.selectedQuoteId == "eth")
        #expect(model.buttonModel.buttonAction == .confirm)
    }

    @Test
    func quoteAskingForDataCollectsItBeforeConfirming() throws {
        let model = model(quotes: .mock(quotes: [.mock(id: "eth", collectDataUrl: "https://data.walletconnect.com/ic/pay_1")]))

        #expect(model.buttonModel.buttonAction == .collectData)
        #expect(model.verificationText != nil)

        model.onSelectVerificationInfo()
        #expect(model.isPresentingSheet == .info(.identityVerification(merchant: "Merchant")))

        model.onSelectButton()
        #expect(try model.isPresentingSheet == .dataCollection(#require(URL(string: "https://data.walletconnect.com/ic/pay_1"))))

        model.onCompleteDataCollection()
        #expect(model.isPresentingSheet == nil)
        #expect(model.buttonModel.buttonAction == .confirm)
        #expect(model.verificationText == nil)
    }

    @Test
    func refetchOfASettledPaymentClosesTheScene() async {
        let completed = Completion()
        let service = PaymentServiceMock(options: .outcome(PaymentOutcome(status: .succeeded, transactionId: "0xhash")))
        let model = model(service: service, onComplete: { completed.value = true })

        await model.fetch()

        #expect(completed.value)
        #expect(model.state.error == nil)
    }

    @Test
    func failedRefetchShowsAnErrorAndRecovers() async {
        let service = PaymentServiceMock(options: .none)
        let model = model(service: service)

        await model.fetch()
        #expect(model.state.refresh.isError)
        #expect(model.buttonModel.buttonAction == .tryAgain)

        service.options = .quotes(.mock(quotes: [.mock(id: "eth")]))
        await model.fetch()

        #expect(model.buttonModel.buttonAction == .confirm)
        #expect(model.state.selectedQuoteId == "eth")
    }

    @Test
    func leavingTheSceneDoesNotExpireThePayment() async {
        let model = model(quotes: .mock(quotes: [.mock(id: "eth")], expiresAt: .now.addingTimeInterval(600)))

        let expiry = Task { await model.awaitExpiry() }
        try? await Task.sleep(for: .milliseconds(10))
        expiry.cancel()
        await expiry.value

        #expect(!model.state.isExpired)
        #expect(model.buttonModel.buttonAction == .confirm)
    }

    @Test
    func failureToPrepareKeepsTheQuotes() async {
        let model = model(quotes: .mock(quotes: [.mock(id: "eth")]))

        await model.confirm()

        #expect(model.state.transferData.isError)
        #expect(model.state.quotes.quotes.count == 1)
        #expect(model.buttonModel.buttonAction == .tryAgain)
    }
}

// MARK: - Private

extension PaymentSceneViewModelTests {
    private func model(quotes: PaymentQuotes) -> PaymentSceneViewModel {
        model(service: PaymentServiceMock(options: .quotes(quotes)), quotes: quotes)
    }

    private func model(
        service: PaymentServiceMock,
        quotes: PaymentQuotes = .mock(quotes: [.mock(id: "eth")]),
        onComplete: VoidAction = .none,
    ) -> PaymentSceneViewModel {
        PaymentSceneViewModel(
            wallet: .mock(),
            link: .walletConnectPay("pay_1"),
            quotes: quotes,
            paymentService: service,
            balanceService: .mock(),
            onTransferAction: .none,
            onComplete: onComplete,
        )
    }
}

@MainActor
private final class Completion {
    var value = false
}

private final class PaymentServiceMock: PaymentServiceable, @unchecked Sendable {
    var options: PaymentOptions?

    init(options: PaymentOptions?) {
        self.options = options
    }

    func getOptions(link _: PaymentLink, addresses _: [ChainAddress]) async throws -> PaymentOptions {
        guard let options else { throw AnyError("no options") }
        return options
    }

    func getQuoteData(quote _: PaymentQuote, addresses _: [ChainAddress]) async throws -> PaymentQuoteData {
        throw AnyError("gateway rejected the quote")
    }

    func confirm(payment _: PaymentData, transactionHash _: String) async throws -> PaymentOutcome {
        PaymentOutcome(status: .succeeded, transactionId: .none)
    }
}

private extension PaymentQuotes {
    static func mock(quotes: [PaymentQuote], expiresAt: Date? = .none) -> PaymentQuotes {
        PaymentQuotes(
            merchant: PaymentMerchant(name: "Merchant", iconUrl: .none),
            price: .none,
            expiresAt: expiresAt,
            quotes: quotes,
        )
    }
}

private extension PaymentQuote {
    static func mock(id: String, collectDataUrl: String? = .none) -> PaymentQuote {
        PaymentQuote(
            id: id,
            link: .walletConnectPay("pay_1"),
            assetId: AssetId(chain: .ethereum, tokenId: .none),
            value: "1000",
            expiresAt: .none,
            collectDataUrl: collectDataUrl,
            providerData: "{}",
        )
    }
}
