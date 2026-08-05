// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import PaymentService
import PrimitivesComponents
import SigningRequestService

public protocol PaymentSheetPresentable: SigningRequestInteractable {
    func collectPaymentData(request: PaymentDataCollectionRequest) async throws -> String
    func selectPaymentQuote(request: PaymentQuotesRequest) async throws -> String
}

@Observable
public final class PaymentSheetPresenter: PaymentSheetPresentable, SheetPresenting, Sendable {
    public let sheets = SheetPresenter<PaymentSheetType>()

    public init() {}

    public func signMessage(payload: SignMessagePayload) async throws -> String {
        try await present(payload: payload) { .signMessage($0) }
    }

    public func signTransaction(transferData: SigningTransferData) async throws -> String {
        try await present(payload: transferData) { .confirm($0) }
    }

    public func sendTransaction(transferData: SigningTransferData) async throws -> String {
        try await present(payload: transferData) { .confirm($0) }
    }

    public func selectPaymentQuote(request: PaymentQuotesRequest) async throws -> String {
        try await present(payload: request) { .quotes($0) }
    }

    public func collectPaymentData(request: PaymentDataCollectionRequest) async throws -> String {
        try await present(payload: request) { .dataCollection($0) }
    }
}
