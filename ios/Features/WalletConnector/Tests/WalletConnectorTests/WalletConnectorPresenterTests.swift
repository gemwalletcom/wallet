import Foundation
import struct Gemstone.SignMessage
import Primitives
import PrimitivesTestKit
import Testing
@testable import WalletConnector
import WalletConnectorService

struct WalletConnectorPresenterTests {
    @Test
    @MainActor
    func completeDismissesSignMessageSheet() {
        let presenter = WalletConnectorPresenter()
        let type = WalletConnectorSheetType.signMessage(
            TransferDataCallback(
                payload: SignMessagePayload(
                    chain: .ethereum,
                    session: .mock(),
                    wallet: .mock(),
                    message: SignMessage(chain: "ethereum", signType: .eip191, data: Data("test".utf8)),
                    simulation: .mock(),
                ),
                delegate: { _ in },
            ),
        )

        presenter.isPresentingSheet = type
        presenter.complete(type: type)

        #expect(presenter.isPresentingSheet == nil)
    }
}
