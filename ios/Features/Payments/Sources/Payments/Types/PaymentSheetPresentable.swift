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
public final class PaymentSheetPresenter: PaymentSheetPresentable, SigningRequestSheetPresentable, Sendable {
    public let sheets = SheetPresenter<PaymentSheetType>()

    public init() {}

    public static func signMessageSheet(_ callback: SigningRequestCallback<SignMessagePayload>) -> PaymentSheetType {
        .signMessage(callback)
    }

    public static func transferSheet(_ callback: SigningRequestCallback<SigningTransferData>) -> PaymentSheetType {
        .confirm(callback)
    }

    public func selectPaymentQuote(request: PaymentQuotesRequest) async throws -> String {
        try await sheets.present(payload: request, sheet: { .quotes($0) })
    }

    public func collectPaymentData(request: PaymentDataCollectionRequest) async throws -> String {
        try await sheets.present(payload: request, sheet: { .dataCollection($0) })
    }
}
