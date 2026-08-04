// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import PaymentService
@testable import Payments
import Primitives
import SigningRequestService

final class PaymentSheetPresentableMock: PaymentSheetPresentable, @unchecked Sendable {
    init() {}

    var collectDataError: Error?
    var selectedQuoteId: String?

    private(set) var collectDataRequests: [PaymentDataCollectionRequest] = []
    private(set) var quotesRequests: [PaymentQuotesRequest] = []

    func selectPaymentQuote(request: PaymentQuotesRequest) async throws -> String {
        quotesRequests.append(request)
        guard let selectedQuoteId else {
            throw AnyError("no quote selected")
        }
        return selectedQuoteId
    }

    func collectPaymentData(request: PaymentDataCollectionRequest) async throws -> String {
        collectDataRequests.append(request)
        if let collectDataError {
            throw collectDataError
        }
        return .empty
    }

    var transactionHash = "1"
    var signature = "signature"
    private(set) var signMessagePayloads: [SignMessagePayload] = []
    private(set) var sentTransferData: [SigningTransferData] = []

    func signMessage(payload: SignMessagePayload) async throws -> String {
        signMessagePayloads.append(payload)
        return signature
    }

    func signTransaction(transferData: SigningTransferData) async throws -> String {
        transactionHash
    }

    func sendTransaction(transferData: SigningTransferData) async throws -> String {
        sentTransferData.append(transferData)
        return transactionHash
    }
}
