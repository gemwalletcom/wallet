import enum Gemstone.GemServiceError
import GemstonePrimitivesTestKit
import Primitives
import PrimitivesTestKit
import Testing
@testable import WalletConnector

@MainActor
struct WalletConnectorPresenterTests {
    @Test
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
    func onlyItsOwnExpiryEndsTheReviewAsExpired() async {
        let presenter = WalletConnectorPresenter()

        await confirmation { answered in
            presenter.present(.transferData(
                TransferDataCallback(
                    payload: WCTransferData(.mock(requestId: "1")),
                    delegate: { result in
                        #expect(throws: GemServiceError.Expired) { try result.get() }
                        answered()
                    },
                ),
            ))

            presenter.expire(requestId: "2")
            #expect(presenter.isPresentingSheet != nil)

            presenter.expire(requestId: "1")
            #expect(presenter.isPresentingSheet == nil)
        }
    }

    @Test
    func presentKeepsRedeliveredProposal() async {
        let presenter = WalletConnectorPresenter()

        await confirmation(expectedCount: 0) { answered in
            presenter.present(.connectionProposal(TransferDataCallback(payload: .mock(pairingId: "pairing"), delegate: { _ in answered() })))
            presenter.present(.connectionProposal(TransferDataCallback(payload: .mock(pairingId: "pairing"), delegate: { _ in answered() })))
        }

        #expect(presenter.isPresentingSheet?.id == AnyHashable("pairing"))
    }

    @Test
    func presentingTheProposalEndsTheConnectionBar() {
        let presenter = WalletConnectorPresenter()
        presenter.isPresentingConnectionBar = true

        presenter.present(.connectionProposal(TransferDataCallback(payload: .mock(pairingId: "pairing"), delegate: { _ in })))

        #expect(presenter.isPresentingConnectionBar == false)
    }

    @Test
    func aRejectedConnectionEndsTheConnectionBar() {
        let presenter = WalletConnectorPresenter()
        presenter.isPresentingConnectionBar = true

        presenter.presentError("No supported wallets are available.")

        #expect(presenter.isPresentingConnectionBar == false)
        #expect(presenter.isPresentingError == "No supported wallets are available.")
    }
}
