// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives
@testable import PrimitivesComponents
import Testing

struct SheetPresenterTests {
    @Test
    @MainActor
    func completeDismissesThePresentedSheet() {
        let presenter = TestPresenter()
        let type = TestSheetType.request(SheetCallback(payload: TestPayload(id: "request"), delegate: { _ in }))

        presenter.sheets.isPresentingSheet = type
        presenter.complete(type: type)

        #expect(presenter.sheets.isPresentingSheet == nil)
    }

    @Test
    @MainActor
    func presentReturnsTheAnswerOnlyAfterTheSheetReportsItClosed() async throws {
        let presenter = TestPresenter()
        let answer = Task { @MainActor in
            try await presenter.present(payload: TestPayload(id: "request")) { .request($0) }
        }
        try await Self.wait { presenter.sheets.isPresentingSheet != nil }

        guard case let .request(callback) = presenter.sheets.isPresentingSheet else {
            Issue.record("sheet is not presented")
            return
        }
        callback.delegate(.success("signature"))
        try await Self.wait { presenter.sheets.isPresentingSheet == nil }
        presenter.onSheetDismiss()

        #expect(try await answer.value == "signature")
    }

    @Test
    @MainActor
    func presentQueuesBehindTheSheetThatIsStillClosing() async throws {
        let presenter = TestPresenter()
        let first = Task { @MainActor in
            try await presenter.present(payload: TestPayload(id: "first")) { .request($0) }
        }
        try await Self.wait { presenter.sheets.isPresentingSheet?.id == "first" }

        guard case let .request(callback) = presenter.sheets.isPresentingSheet else {
            Issue.record("sheet is not presented")
            return
        }
        let second = Task { @MainActor in
            try await presenter.present(payload: TestPayload(id: "second")) { .request($0) }
        }
        callback.delegate(.success("signature"))
        try await Self.wait { presenter.sheets.isPresentingSheet == nil }

        #expect(presenter.sheets.isPresentingSheet == nil)

        presenter.onSheetDismiss()
        try await Self.wait { presenter.sheets.isPresentingSheet?.id == "second" }

        #expect(try await first.value == "signature")

        guard let sheet = presenter.sheets.isPresentingSheet else {
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
        let presenter = TestPresenter()
        let answer = Task { @MainActor in
            try await presenter.present(payload: TestPayload(id: "request")) { .request($0) }
        }
        try await Self.wait { presenter.sheets.isPresentingSheet != nil }

        guard let sheet = presenter.sheets.isPresentingSheet else {
            Issue.record("sheet is not presented")
            return
        }
        presenter.cancelSheet(type: sheet)
        presenter.onSheetDismiss()

        await #expect(throws: SigningRequestError.userCancelled) {
            try await answer.value
        }
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

private struct TestPayload: Identifiable, Sendable {
    let id: String
}

private struct TestPresenter: SheetPresenting {
    let sheets = SheetPresenter<TestSheetType>()
}

private enum TestSheetType: Sendable, Identifiable, SheetRejectable {
    case request(SheetCallback<TestPayload>)

    var id: String {
        switch self {
        case let .request(callback): callback.id
        }
    }

    func reject(_ error: any Error) {
        switch self {
        case let .request(callback): callback.reject(error)
        }
    }
}
