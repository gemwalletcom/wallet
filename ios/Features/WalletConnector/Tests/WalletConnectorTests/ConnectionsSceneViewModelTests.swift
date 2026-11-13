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
        service.connectionGroupsValue = [
            GemGroup(id: "wallet-1", title: .text(text: "Active"), footer: .none, items: [.connection(connection: GemConnection(connection: connection.toGem(), row: service.connectionRowValue))]),
        ]
        let model = ConnectionsSceneViewModel.mock(service: service)
        model.query.value = [connection]

        #expect(model.view.list.groups.map { $0.title?.text } == ["Active"])
        #expect(service.viewedConnections == [connection.toGem()])
        #expect(model.view.list.groups.first?.items.count == 1)
        #expect(model.view.list.phase == .rows)
    }

    @Test
    func noConnectionsMeanNoSections() {
        let service = GemWalletConnectServiceMock()
        let model = ConnectionsSceneViewModel.mock(service: service)

        #expect(model.view.list.groups.isEmpty)
        #expect(service.viewedConnections.isEmpty)
        #expect(model.view.list.phase == .empty(state: emptyState(kind: .walletConnect)))
        #expect(model.connections.isEmpty)
    }

    @Test
    func theDetailsSceneReadsCore() {
        let service = GemWalletConnectServiceMock()
        service.connectionDetailRows = [.text(title: .wallet, value: "Main Wallet")]
        let model = ConnectionsSceneViewModel.mock(service: service)
        let connection = WalletConnection.mock(session: .mock(sessionId: "topic"))
        model.query.value = [.mock(session: .mock(sessionId: "other")), connection]

        let details = model.connectionDetails(connection: connection)

        #expect(details.rows == [.text(title: .wallet, value: "Main Wallet")])
        #expect(details.connection.connection == connection.toGem())
    }

    @Test
    func scanningOpensTheScanner() {
        let model = ConnectionsSceneViewModel.mock()

        model.onScan()

        #expect(model.isPresentingScanner)
    }

    @Test
    func aScannedUriPairsAndShowsTheConnectorBar() async {
        let uri = "wc:abc@2?relay-protocol=irn&symKey=123"
        let connector = WalletConnectorServiceMock()
        let presenter = WalletConnectorPresenter()
        let model = ConnectionsSceneViewModel.mock(connector: connector, walletConnectorPresenter: presenter)

        await model.onHandleScan(uri)

        #expect(await connector.pairedURIs == [uri])
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
