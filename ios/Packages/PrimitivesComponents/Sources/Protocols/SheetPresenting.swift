// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives

public protocol SheetPresenting: Sendable {
    associatedtype Sheet: SheetRejectable & Identifiable where Sheet.ID == String

    var sheets: SheetPresenter<Sheet> { get }
}

public extension SheetPresenting {
    @MainActor
    var isPresentingSheet: Sheet? {
        get { sheets.isPresentingSheet }
        nonmutating set { sheets.isPresentingSheet = newValue }
    }

    @MainActor
    func complete(type: Sheet) {
        sheets.dismiss(id: type.id)
    }

    @MainActor
    func cancelSheet(type: Sheet) {
        guard sheets.isPresentingSheet?.id == type.id else {
            return
        }
        type.reject(SigningRequestError.userCancelled)
        sheets.dismiss(id: type.id)
    }

    @MainActor
    func onSheetDismiss() {
        sheets.onSheetDismiss()
    }

    func present<Payload: Identifiable & Sendable>(
        payload: Payload,
        sheet: @Sendable @escaping (SheetCallback<Payload>) -> Sheet,
    ) async throws -> String where Payload.ID == String {
        let (stream, continuation) = AsyncThrowingStream.makeStream(of: String.self)
        let callback = SheetCallback(payload: payload) {
            continuation.yield(with: $0)
            continuation.finish()
        }
        await sheets.show(sheet: sheet(callback))

        do {
            for try await value in stream {
                await sheets.dismissPresented()
                return value
            }
        } catch {
            await sheets.dismissPresented()
            throw error
        }
        await sheets.dismissPresented()
        throw SigningRequestError.userCancelled
    }
}
