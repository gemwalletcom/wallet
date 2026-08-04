// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import PaymentService
import Primitives
import SigningRequestService

public protocol PaymentSheetPresentable: SigningRequestInteractable {
    func collectPaymentData(request: PaymentDataCollectionRequest) async throws -> String
    func selectPaymentQuote(request: PaymentQuotesRequest) async throws -> String
}

@Observable
public final class PaymentSheetPresenter: PaymentSheetPresentable, Sendable {
    public let sheets = SheetPresenter<PaymentSheetType>()

    public init() {}

    @MainActor
    public var isPresentingSheet: PaymentSheetType? {
        get { sheets.isPresentingSheet }
        set { sheets.isPresentingSheet = newValue }
    }

    @MainActor
    public func complete(type: PaymentSheetType) {
        sheets.complete(type: type)
    }

    @MainActor
    public func cancelSheet(type: PaymentSheetType) {
        sheets.cancelSheet(type: type)
    }

    @MainActor
    public func onSheetDismiss() {
        sheets.onSheetDismiss()
    }

    public func selectPaymentQuote(request: PaymentQuotesRequest) async throws -> String {
        try await sheets.present(payload: request, sheet: { .quotes($0) })
    }

    public func collectPaymentData(request: PaymentDataCollectionRequest) async throws -> String {
        try await sheets.present(payload: request, sheet: { .dataCollection($0) })
    }

    public func signMessage(payload: SignMessagePayload) async throws -> String {
        try await sheets.present(payload: payload, sheet: { .signMessage($0) })
    }

    public func signTransaction(transferData: SigningTransferData) async throws -> String {
        try await sheets.present(payload: transferData, sheet: { .confirm($0) })
    }

    public func sendTransaction(transferData: SigningTransferData) async throws -> String {
        try await sheets.present(payload: transferData, sheet: { .confirm($0) })
    }
}
