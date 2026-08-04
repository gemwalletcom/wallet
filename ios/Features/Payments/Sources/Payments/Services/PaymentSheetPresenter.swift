// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import PaymentService
import Primitives

public protocol PaymentSheetPresentable: Sendable {
    func collectPaymentData(request: PaymentDataCollectionRequest) async throws -> String
    func selectPaymentQuote(request: PaymentQuotesRequest) async throws -> String
}

extension SheetPresenter: PaymentSheetPresentable where Sheet == PaymentSheetType {
    public func selectPaymentQuote(request: PaymentQuotesRequest) async throws -> String {
        try await present(payload: request, sheet: { .quotes($0) })
    }

    public func collectPaymentData(request: PaymentDataCollectionRequest) async throws -> String {
        try await present(payload: request, sheet: { .dataCollection($0) })
    }
}
