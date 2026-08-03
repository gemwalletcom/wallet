// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import PaymentService
import PaymentServiceTestKit
@testable import Payments
import Primitives
import PrimitivesTestKit
import SigningRequestService
import Testing

struct PaymentSheetPresenterTests {
    @Test
    @MainActor
    func selectPaymentQuoteReturnsTheSelectionOnceTheSheetHasClosed() async throws {
        let presenter = PaymentSheetPresenter()
        let selection = Task { @MainActor in
            try await presenter.selectPaymentQuote(request: Self.request())
        }
        try await Self.wait { presenter.isPresentingSheet != nil }

        guard case let .quotes(callback) = presenter.isPresentingSheet else {
            Issue.record("quotes sheet is not presented")
            return
        }
        callback.delegate(.success("quote_2"))
        try await Self.wait { presenter.isPresentingSheet == nil }
        presenter.onSheetDismiss()

        #expect(try await selection.value == "quote_2")
    }

    @Test
    @MainActor
    func cancelSheetFailsTheRequestWithUserCancelled() async throws {
        let presenter = PaymentSheetPresenter()
        let selection = Task { @MainActor in
            try await presenter.selectPaymentQuote(request: Self.request())
        }
        try await Self.wait { presenter.isPresentingSheet != nil }

        guard let sheet = presenter.isPresentingSheet else {
            Issue.record("quotes sheet is not presented")
            return
        }
        presenter.cancelSheet(type: sheet)
        presenter.onSheetDismiss()

        await #expect(throws: SigningRequestError.userCancelled) {
            try await selection.value
        }
    }

    private static func request() -> PaymentQuotesRequest {
        PaymentQuotesRequest(id: "pay_1", quotes: .mock(), wallet: .mock(), assetsData: [])
    }

    private static func wait(until condition: @MainActor () -> Bool) async throws {
        for _ in 0 ..< 100 {
            if await condition() {
                return
            }
            await Task.yield()
        }
        throw AnyError("condition never became true")
    }
}
