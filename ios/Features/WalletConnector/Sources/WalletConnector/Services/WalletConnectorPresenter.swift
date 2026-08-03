// Copyright (c). Gem Wallet. All rights reserved.

import PaymentService
import SigningRequestService
import SwiftUI

@Observable
public final class WalletConnectorPresenter: Sendable {
    @MainActor
    public var isPresentingError: String?
    @MainActor
    public var isPresentingConnectionBar: Bool = false
    @MainActor
    public var isPresentingSheet: WalletConnectorSheetType?
    @MainActor
    private var isDismissingSheet: Bool = false
    @MainActor
    private var dismissals: [CheckedContinuation<Void, Never>] = []

    public init() {}

    @MainActor
    public func present(sheet: WalletConnectorSheetType) async {
        await waitForDismiss()
        isPresentingSheet = sheet
    }

    @MainActor
    public func dismissPresentedSheet() async {
        dismiss()
        await waitForDismiss()
    }

    @MainActor
    public func complete(type: WalletConnectorSheetType) {
        guard isPresentingSheet?.id == type.id else {
            return
        }
        dismiss()
    }

    @MainActor
    public func cancelSheet(type: WalletConnectorSheetType) {
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

extension WalletConnectorPresenter {
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
