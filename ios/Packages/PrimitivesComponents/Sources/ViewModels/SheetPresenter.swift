// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

@Observable
public final class SheetPresenter<Sheet: SheetRejectable & Identifiable>: Sendable where Sheet.ID == String {
    @MainActor
    public var isPresentingSheet: Sheet?
    @MainActor
    private var isDismissingSheet: Bool = false
    @MainActor
    private var dismissals: [CheckedContinuation<Void, Never>] = []

    public init() {}

    @MainActor
    public func onSheetDismiss() {
        isDismissingSheet = false
        let waiting = dismissals
        dismissals = []
        waiting.forEach { $0.resume() }
    }
}

// MARK: - Internal

extension SheetPresenter {
    @MainActor
    func show(sheet: Sheet) async {
        await waitForDismiss()
        isPresentingSheet = sheet
    }

    @MainActor
    func dismissPresented() async {
        dismiss()
        await waitForDismiss()
    }

    @MainActor
    func dismiss(id: String) {
        guard isPresentingSheet?.id == id else {
            return
        }
        dismiss()
    }
}

// MARK: - Private

extension SheetPresenter {
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
