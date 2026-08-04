// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives
import PrimitivesTestKit
@testable import SigningRequestService
import SigningRequestServiceTestKit
import Testing

struct SheetPresenterTests {
    @Test
    @MainActor
    func completeDismissesThePresentedSheet() {
        let presenter = SheetPresenter<TestSheetType>()
        let type = Self.sheet(id: "request")

        presenter.isPresentingSheet = type
        presenter.complete(type: type)

        #expect(presenter.isPresentingSheet == nil)
    }

    @Test
    @MainActor
    func presentReturnsTheAnswerOnlyAfterTheSheetReportsItClosed() async throws {
        let presenter = SheetPresenter<TestSheetType>()
        let answer = Task { @MainActor in
            try await presenter.present(payload: SignMessagePayload.mock(id: "request"), sheet: { .signMessage($0) })
        }
        try await Self.wait { presenter.isPresentingSheet != nil }

        guard case let .signMessage(callback) = presenter.isPresentingSheet else {
            Issue.record("sheet is not presented")
            return
        }
        callback.delegate(.success("signature"))
        try await Self.wait { presenter.isPresentingSheet == nil }
        presenter.onSheetDismiss()

        #expect(try await answer.value == "signature")
    }

    @Test
    @MainActor
    func presentQueuesBehindTheSheetThatIsStillClosing() async throws {
        let presenter = SheetPresenter<TestSheetType>()
        let first = Task { @MainActor in
            try await presenter.present(payload: SignMessagePayload.mock(id: "first"), sheet: { .signMessage($0) })
        }
        try await Self.wait { presenter.isPresentingSheet?.id == "first" }

        guard case let .signMessage(callback) = presenter.isPresentingSheet else {
            Issue.record("sheet is not presented")
            return
        }
        let second = Task { @MainActor in
            try await presenter.present(payload: SignMessagePayload.mock(id: "second"), sheet: { .signMessage($0) })
        }
        callback.delegate(.success("signature"))
        try await Self.wait { presenter.isPresentingSheet == nil }

        #expect(presenter.isPresentingSheet == nil)

        presenter.onSheetDismiss()
        try await Self.wait { presenter.isPresentingSheet?.id == "second" }

        #expect(try await first.value == "signature")

        guard let sheet = presenter.isPresentingSheet else {
            Issue.record("queued sheet is not presented")
            return
        }
        presenter.cancelSheet(type: sheet)
        presenter.onSheetDismiss()
        _ = try? await second.value
    }

    @Test
    @MainActor
    func cancelSheetFailsTheRequestWithUserCancelled() async throws {
        let presenter = SheetPresenter<TestSheetType>()
        let answer = Task { @MainActor in
            try await presenter.present(payload: SignMessagePayload.mock(id: "request"), sheet: { .signMessage($0) })
        }
        try await Self.wait { presenter.isPresentingSheet != nil }

        guard let sheet = presenter.isPresentingSheet else {
            Issue.record("sheet is not presented")
            return
        }
        presenter.cancelSheet(type: sheet)
        presenter.onSheetDismiss()

        await #expect(throws: SigningRequestError.userCancelled) {
            try await answer.value
        }
    }

    private static func sheet(id: String) -> TestSheetType {
        .signMessage(SigningRequestCallback(payload: .mock(id: id), delegate: { _ in }))
    }

    private static func wait(until condition: @MainActor () -> Bool) async throws {
        for _ in 0 ..< 100 {
            if await condition() {
                return
            }
            await Task.yield()
        }
        throw AnyError("condition never became true")
    }
}

private enum TestSheetType: Sendable, Identifiable, SigningRequestRejectable {
    case signMessage(SigningRequestCallback<SignMessagePayload>)

    var id: String {
        switch self {
        case let .signMessage(callback): callback.id
        }
    }

    func reject(_ error: any Error) {
        switch self {
        case let .signMessage(callback): callback.reject(error)
        }
    }
}
