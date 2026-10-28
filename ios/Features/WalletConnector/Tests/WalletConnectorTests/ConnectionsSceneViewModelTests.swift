// Copyright (c). Gem Wallet. All rights reserved.

import Gemstone
import GemstonePrimitives
import GemstonePrimitivesTestKit
import Primitives
import PrimitivesTestKit
@testable import Store
import Testing
@testable import WalletConnector
import WalletConnectorServiceTestKit
import WalletConnectorTestKit

@MainActor
struct ConnectionsSceneViewModelTests {
    @Test
    func theSectionsComeFromCore() {
        let service = GemWalletConnectServiceMock()
        let connection = WalletConnection.mock()
        service.connectionSectionsValue = [
            GemConnectionSection(title: "Active", connections: [GemConnection(connection: connection.toGem(), row: service.connectionRowValue)]),
        ]
        let model = ConnectionsSceneViewModel.mock(service: service)
        model.query.value = [connection]

        #expect(model.view.sections.map(\.title) == ["Active"])
        #expect(model.view.sections.first?.connections.count == 1)
        #expect(model.view.phase == .rows)
    }

    @Test
    func noConnectionsMeanNoSections() {
        let model = ConnectionsSceneViewModel.mock()

        #expect(model.view.sections.isEmpty)
        #expect(model.view.phase == .empty(state: emptyState(kind: .walletConnect)))
        #expect(model.connections.isEmpty)
    }

    @Test
    func theDetailsSceneReadsCore() {
        let service = GemWalletConnectServiceMock()
        service.connectionDetailRows = [.text(title: .wallet, value: "Main Wallet")]
        let model = ConnectionsSceneViewModel.mock(service: service)

        let details = model.connectionDetails(connection: .mock())

        #expect(details.rows == [.text(title: .wallet, value: "Main Wallet")])
    }

    @Test
    func scanningOpensTheScanner() {
        let model = ConnectionsSceneViewModel.mock()

        model.onScan()

        #expect(model.isPresentingScanner)
    }

    @Test
    func aScannedUriPairsAndShowsTheConnectorBar() async {
        let presenter = WalletConnectorPresenter()
        let model = ConnectionsSceneViewModel.mock(walletConnectorPresenter: presenter)

        await model.onHandleScan("wc:topic@2")

        #expect(presenter.isPresentingConnectionBar)
        #expect(model.isPresentingAlertMessage == nil)
    }

    @Test
    func aFailedPairHidesTheBarAndShowsTheError() async {
        let presenter = WalletConnectorPresenter()
        let model = ConnectionsSceneViewModel.mock(connector: WalletConnectorServiceMock(pairError: AnyError("bad uri")), walletConnectorPresenter: presenter)

        await model.onHandleScan("nonsense")

        #expect(presenter.isPresentingConnectionBar == false)
        #expect(model.isPresentingAlertMessage?.message == "bad uri")
    }

    @Test
    func aFailedDisconnectShowsTheError() async {
        let model = ConnectionsSceneViewModel.mock(connector: WalletConnectorServiceMock(disconnectError: AnyError("no session")))

        await model.onSelectDisconnect(.mock())

        #expect(model.isPresentingAlertMessage?.message == "no session")
    }
}
