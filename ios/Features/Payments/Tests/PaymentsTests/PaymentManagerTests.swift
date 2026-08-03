// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Localization
import PaymentService
import Primitives
import SigningRequestService
import PrimitivesTestKit
import Store
import StoreTestKit
import PaymentServiceTestKit
import SigningRequestServiceTestKit
import Testing
import TransactionStateServiceTestKit
@testable import Payments

@MainActor
struct PaymentManagerTests {
    private let interactor = SigningRequestInteractableMock()
    private let presenter = PaymentSheetPresentableMock()

    private func makeManager(
        service: PaymentServiceableMock,
        transactionStore: TransactionStore = .mock(),
    ) -> PaymentManager {
        PaymentManager(
            service: service,
            executor: PaymentActionExecutor(interactor: interactor, simulator: SigningSimulatableMock(), approvalExecutor: PaymentApprovalExecutableMock(), assetsProvider: PaymentAssetsProvidableMock()),
            presenter: presenter,
            assetsProvider: PaymentAssetsProvidableMock(),
            transactionStateScheduler: .mock(
                transactionStore: transactionStore,
                paymentStatusService: PaymentStatusServiceableMock(result: .mock(status: .processing)),
            ),
        )
    }

    @Test
    func payReturnsSettledPaymentWithoutSigning() async throws {
        let service = PaymentServiceableMock(options: [.outcome(.mock())])

        let outcome = try await makeManager(service: service).pay(link: .mock(), wallet: .mock())

        #expect(outcome.status == .succeeded)
        #expect(interactor.signMessagePayloads.isEmpty)
        #expect(await service.confirmedResults.isEmpty)
    }

    @Test
    func paySignsActionsAndConfirms() async throws {
        let service = PaymentServiceableMock(
            options: [.quotes(.mock(merchant: .mock(name: "Coffee Shop")))],
            actions: [.mockSignMessage(data: Data("pay".utf8))],
        )

        let outcome = try await makeManager(service: service).pay(link: .mock(), wallet: .mock())

        #expect(interactor.signMessagePayloads.first?.id == "pay_1.0")
        #expect(interactor.signMessagePayloads.first?.appMetadata.name == "Coffee Shop")
        #expect(await service.confirmedResults == [["signature"]])
        #expect(outcome.status == .succeeded)
    }
    @Test
    func payStaysPendingWhenConfirmFails() async throws {
        let service = PaymentServiceableMock(
            options: [.quotes(.mock())],
            actions: [.mockSignMessage(data: Data("pay".utf8))],
            confirmError: AnyError("gateway timeout"),
        )

        let outcome = try await makeManager(service: service).pay(link: .mock(), wallet: .mock())

        #expect(outcome.status == .processing)
    }

    @Test
    func payCollectsDataThenSigns() async throws {
        let quote = PaymentQuote.mock(collectDataUrl: "https://data-collection.walletconnect.com/ic/pay_1")
        let service = PaymentServiceableMock(options: [.quotes(.mock(quotes: [quote]))])

        _ = try await makeManager(service: service).pay(link: .mock(), wallet: .mock())

        #expect(presenter.collectDataRequests.first?.url.absoluteString == "https://data-collection.walletconnect.com/ic/pay_1")
        #expect(await service.confirmedResults == [[]])
    }

    @Test
    func paySignsWithoutCollectingDataWhenQuoteDoesNotAskForIt() async throws {
        let service = PaymentServiceableMock(options: [.quotes(.mock())])

        _ = try await makeManager(service: service).pay(link: .mock(), wallet: .mock())

        #expect(presenter.collectDataRequests.isEmpty)
        #expect(await service.confirmedResults == [[]])
    }
    @Test
    func payKeepsThePaymentAliveWhenUserClosesDataCollection() async throws {
        presenter.collectDataError = SigningRequestError.userCancelled
        let quote = PaymentQuote.mock(collectDataUrl: "https://data-collection.walletconnect.com/ic/pay_1")
        let service = PaymentServiceableMock(options: [.quotes(.mock(quotes: [quote]))])

        let outcome = try await makeManager(service: service).pay(link: .mock(), wallet: .mock())

        #expect(outcome.status == .cancelled)
        #expect(await service.cancelledPaymentIds.isEmpty)
        #expect(await service.confirmedResults.isEmpty)
    }

    @Test
    func payRecordsPendingPaymentBeforeConfirming() async throws {
        let assetId = AssetId.mock(.ethereum)
        let store = TransactionStore.mock(db: .mockAssets(assets: [.mock(asset: .mock(id: assetId))]))
        let service = PaymentServiceableMock(options: [.quotes(.mock(quotes: [.mock(amount: .mock(assetId: assetId))]))])

        _ = try await makeManager(service: service, transactionStore: store)
            .pay(link: .mock(), wallet: .mock(accounts: [.mock(chain: assetId.chain)]))

        let saved = try #require(store.getTransactions(states: [.pending]).first)
        #expect(saved.id.hash == "pay_1")
        #expect(saved.value == "10000")
        #expect(saved.metadata?.decode(TransactionPaymentMetadata.self)?.merchant.name == "Test Merchant")
        #expect(await service.confirmedResults == [[]])
    }

    @Test
    func payUsesTheQuoteTheBuyerPicked() async throws {
        let other = PaymentQuote.mock(amount: .mock(symbol: "USDT"), id: "option_2")
        let service = PaymentServiceableMock(
            options: [.quotes(.mock(quotes: [.mock(), other]))],
            actions: [.mockSignMessage(data: Data("pay".utf8))],
        )
        presenter.selectedQuoteId = other.id

        _ = try await makeManager(service: service).pay(link: .mock(), wallet: .mock())

        #expect(presenter.quotesRequests.count == 1)
        #expect(interactor.signMessagePayloads.count == 1)
        #expect(interactor.signMessagePayloads.first?.payment?.quote == other)
    }

    @Test
    func payDoesNotAskWhichQuoteWhenThereIsOnlyOne() async throws {
        let service = PaymentServiceableMock(options: [.quotes(.mock(quotes: [.mock()]))])

        _ = try await makeManager(service: service).pay(link: .mock(), wallet: .mock())

        #expect(presenter.quotesRequests.isEmpty)
        #expect(await service.confirmedResults == [[]])
    }

    @Test
    func payCollectsDataOnlyForTheQuoteTheBuyerPicked() async throws {
        let url = "https://data-collection.walletconnect.com/ic/pay_1"
        let other = PaymentQuote.mock(amount: .mock(symbol: "USDT"), id: "option_2")
        let service = PaymentServiceableMock(options: [.quotes(.mock(quotes: [.mock(collectDataUrl: url), other]))])
        presenter.selectedQuoteId = other.id

        _ = try await makeManager(service: service).pay(link: .mock(), wallet: .mock())

        #expect(presenter.collectDataRequests.isEmpty)
    }

    @Test
    func payReportsUnpayableStatuses() async throws {
        for status in [PaymentStatus.expired, .failed] {
            let service = PaymentServiceableMock(options: [.outcome(.mock(status: status))])

            let outcome = try await makeManager(service: service).pay(link: .mock(), wallet: .mock())

            #expect(outcome.status == status)
            #expect(await service.confirmedResults.isEmpty)
        }
    }
}
