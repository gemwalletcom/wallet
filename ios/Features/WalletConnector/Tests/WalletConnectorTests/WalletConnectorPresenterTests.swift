import Foundation
import struct Gemstone.SignMessage
import Primitives
import SigningRequestService
import PrimitivesTestKit
import SigningRequestServiceTestKit
import Testing
@testable import WalletConnector
import WalletConnectorService

struct WalletConnectorPresenterTests {
    @Test
    @MainActor
    func completeDismissesSignMessageSheet() {
        let presenter = WalletConnectorPresenter()
        let type = Self.sheet(id: "request")

        presenter.isPresentingSheet = type
        presenter.complete(type: type)

        #expect(presenter.isPresentingSheet == nil)
    }

    @Test
    @MainActor
    func dismissPresentedSheetWaitsForTheSheetToReportItClosed() async {
        let presenter = WalletConnectorPresenter()
        presenter.isPresentingSheet = Self.sheet(id: "request")

        let dismissed = Task { @MainActor in
            await presenter.dismissPresentedSheet()
            return true
        }
        await Task.yield()

        #expect(presenter.isPresentingSheet == nil)
        #expect(!dismissed.isCancelled)

        presenter.onSheetDismiss()

        #expect(await dismissed.value)
    }

    @Test
    @MainActor
    func presentWaitsUntilThePreviousSheetHasClosed() async {
        let presenter = WalletConnectorPresenter()
        presenter.isPresentingSheet = Self.sheet(id: "first")

        let dismissed = Task { @MainActor in await presenter.dismissPresentedSheet() }
        await Task.yield()

        let next = Self.sheet(id: "second")
        let presented = Task { @MainActor in await presenter.present(sheet: next) }
        await Task.yield()

        #expect(presenter.isPresentingSheet == nil)

        presenter.onSheetDismiss()
        await dismissed.value
        await presented.value

        #expect(presenter.isPresentingSheet?.id == "second")
    }

    @Test
    @MainActor
    func presentDoesNotWaitWhenNoSheetIsShowing() async {
        let presenter = WalletConnectorPresenter()
        let type = Self.sheet(id: "request")

        await presenter.present(sheet: type)

        #expect(presenter.isPresentingSheet?.id == "request")
    }

    private static func sheet(id: String) -> WalletConnectorSheetType {
        .signMessage(
            SigningRequestCallback(
                payload: SignMessagePayload(
                    id: id,
                    chain: .ethereum,
                    appMetadata: .mock(),
                    wallet: .mock(),
                    message: SignMessage(chain: "ethereum", signType: .eip191, data: Data("test".utf8)),
                    simulation: .mock(),
                ),
                delegate: { _ in },
            ),
        )
    }
}
