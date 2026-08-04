// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

@Observable
public final class SheetPresenter<Sheet: SigningRequestRejectable & Identifiable>: Sendable where Sheet.ID == String {
    @MainActor
    public var isPresentingSheet: Sheet?
    @MainActor
    private var isDismissingSheet: Bool = false
    @MainActor
    private var dismissals: [CheckedContinuation<Void, Never>] = []

    public init() {}

    public func present<Payload: Identifiable & Sendable>(
        payload: Payload,
        sheet: @Sendable @escaping (SigningRequestCallback<Payload>) -> Sheet,
    ) async throws -> String where Payload.ID == String {
        let (stream, continuation) = AsyncThrowingStream.makeStream(of: String.self)
        let callback = SigningRequestCallback(payload: payload) {
            continuation.yield(with: $0)
            continuation.finish()
        }
        await show(sheet: sheet(callback))

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
    public func complete(type: Sheet) {
        guard isPresentingSheet?.id == type.id else {
            return
        }
        dismiss()
    }

    @MainActor
    public func cancelSheet(type: Sheet) {
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

extension SheetPresenter {
    @MainActor
    private func show(sheet: Sheet) async {
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
