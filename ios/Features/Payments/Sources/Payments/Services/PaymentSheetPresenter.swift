// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import PaymentService
import Primitives
import SigningRequestService
import SwiftUI

@Observable
public final class PaymentSheetPresenter: PaymentSheetPresentable, Sendable {
    @MainActor
    public var isPresentingSheet: PaymentSheetType?
    @MainActor
    private var isDismissingSheet: Bool = false
    @MainActor
    private var dismissals: [CheckedContinuation<Void, Never>] = []

    public init() {}

    public func selectPaymentQuote(request: PaymentQuotesRequest) async throws -> String {
        try await present(payload: request, sheet: { .quotes($0) })
    }

    public func collectPaymentData(request: PaymentDataCollectionRequest) async throws -> String {
        try await present(payload: request, sheet: { .dataCollection($0) })
    }

    @MainActor
    public func complete(type: PaymentSheetType) {
        guard isPresentingSheet?.id == type.id else {
            return
        }
        dismiss()
    }

    @MainActor
    public func cancelSheet(type: PaymentSheetType) {
        guard isPresentingSheet?.id == type.id else {
            return
        }
        type.reject(SigningRequestError.userCancelled)
        dismiss()
    }

    @MainActor
    public func onSheetDismiss() {
        isDismissingSheet = false
        let waiting = dismissals
        dismissals = []
        waiting.forEach { $0.resume() }
    }
}

// MARK: - Private

extension PaymentSheetPresenter {
    private func present<T: Identifiable & Sendable>(
        payload: T,
        sheet: @Sendable @escaping (SigningRequestCallback<T>) -> PaymentSheetType,
    ) async throws -> String where T.ID == String {
        let (stream, continuation) = AsyncThrowingStream.makeStream(of: String.self)
        let callback = SigningRequestCallback(payload: payload) {
            continuation.yield(with: $0)
            continuation.finish()
        }
        await present(sheet: sheet(callback))

        do {
            for try await value in stream {
                await dismissPresentedSheet()
                return value
            }
        } catch {
            await dismissPresentedSheet()
            throw error
        }
        await dismissPresentedSheet()
        throw SigningRequestError.userCancelled
    }

    @MainActor
    private func present(sheet: PaymentSheetType) async {
        await waitForDismiss()
        isPresentingSheet = sheet
    }

    @MainActor
    private func dismissPresentedSheet() async {
        dismiss()
        await waitForDismiss()
    }

    @MainActor
    private func dismiss() {
        guard isPresentingSheet != nil else {
            return
        }
        isDismissingSheet = true
        isPresentingSheet = .none
    }

    @MainActor
    private func waitForDismiss() async {
        guard isPresentingSheet != nil || isDismissingSheet else {
            return
        }
        await withCheckedContinuation { dismissals.append($0) }
    }
}
