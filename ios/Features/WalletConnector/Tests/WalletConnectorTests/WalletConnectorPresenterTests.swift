import Primitives
import PrimitivesTestKit
import Testing
@testable import WalletConnector
import WalletConnectorService
import WalletConnectorServiceTestKit

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
}
