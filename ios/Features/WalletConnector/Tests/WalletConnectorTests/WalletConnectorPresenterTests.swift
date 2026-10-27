import Primitives
import PrimitivesTestKit
import Testing
@testable import WalletConnector

struct WalletConnectorPresenterTests {
    @Test
    @MainActor
    func completeDismissesSignMessageSheet() {
        let presenter = WalletConnectorPresenter()
        let type = WalletConnectorSheetType.signMessage(
            TransferDataCallback(
                payload: .mock(),
                delegate: { _ in },
            ),
        )

        presenter.isPresentingSheet = type
        presenter.complete(type: type)

        #expect(presenter.isPresentingSheet == nil)
    }

    @Test
    @MainActor
    func presentRejectsDisplacedRequest() async {
        let presenter = WalletConnectorPresenter()
        let second = WalletConnectorSheetType.signMessage(
            TransferDataCallback(
                payload: .mock(),
                delegate: { _ in Issue.record("second request answered") },
            ),
        )

        await confirmation { rejected in
            presenter.present(.signMessage(
                TransferDataCallback(
                    payload: .mock(),
                    delegate: { result in
                        if case .failure(ConnectionsError.userCancelled) = result { rejected() }
                    },
                ),
            ))
            presenter.present(second)
        }

        #expect(presenter.isPresentingSheet?.id == second.id)
    }

    @Test
    @MainActor
    func presentKeepsRedeliveredProposal() async {
        let presenter = WalletConnectorPresenter()

        await confirmation(expectedCount: 0) { answered in
            presenter.present(.connectionProposal(TransferDataCallback(payload: .mock(pairingId: "pairing"), delegate: { _ in answered() })))
            presenter.present(.connectionProposal(TransferDataCallback(payload: .mock(pairingId: "pairing"), delegate: { _ in answered() })))
        }

        #expect(presenter.isPresentingSheet?.id == AnyHashable("pairing"))
    }

    @Test
    @MainActor
    func presentingTheProposalEndsTheConnectionBar() {
        let presenter = WalletConnectorPresenter()
        presenter.isPresentingConnectionBar = true

        presenter.present(.connectionProposal(TransferDataCallback(payload: .mock(pairingId: "pairing"), delegate: { _ in })))

        #expect(presenter.isPresentingConnectionBar == false)
    }

    @Test
    @MainActor
    func aRejectedConnectionEndsTheConnectionBar() {
        let presenter = WalletConnectorPresenter()
        presenter.isPresentingConnectionBar = true

        presenter.presentError("No supported wallets are available.")

        #expect(presenter.isPresentingConnectionBar == false)
        #expect(presenter.isPresentingError == "No supported wallets are available.")
    }
}
