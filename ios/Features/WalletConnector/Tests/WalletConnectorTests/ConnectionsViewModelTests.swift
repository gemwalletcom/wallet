// Copyright (c). Gem Wallet. All rights reserved.

import Gemstone
import GemstonePrimitives
import GemstonePrimitivesTestKit
import Primitives
import PrimitivesTestKit
@testable import Store
import StoreTestKit
import Testing
@testable import WalletConnector
import WalletConnectorServiceTestKit

@MainActor
struct ConnectionsViewModelTests {
    private func model(
        connector: WalletConnectorServiceMock = WalletConnectorServiceMock(),
        service: GemWalletConnectServiceMock = GemWalletConnectServiceMock(),
    ) -> ConnectionsViewModel {
        ConnectionsViewModel(connector: connector, service: service)
    }

    @Test
    func theSectionsComeFromCore() {
        let service = GemWalletConnectServiceMock()
        let connection = WalletConnection.mock()
        service.connectionSectionsValue = [
            GemConnectionSection(title: "Active", connections: [GemConnection(connection: connection.toGem(), row: service.connectionRowValue)]),
        ]
        let model = model(service: service)
        model.query.value = [connection]

        #expect(model.sections.map(\.title) == ["Active"])
        #expect(model.sections.first?.connections.count == 1)
    }

    @Test
    func noConnectionsMeanNoSections() {
        let model = model()

        #expect(model.sections.isEmpty)
        #expect(model.connections.isEmpty)
    }

    @Test
    func theDetailsSceneReadsCore() {
        let service = GemWalletConnectServiceMock()
        service.connectionDetailRows = [.wallet, .date]
        let model = model(service: service)

        let details = model.connectionSceneModel(connection: .mock())

        #expect(details.details.rows == [.wallet, .date])
    }

    @Test
    func scanningOpensTheScanner() {
        let model = model()

        model.onScan()

        #expect(model.isPresentingScanner)
    }

    @Test
    func aScannedUriPairsAndShowsTheConnectorBar() async {
        let model = model()

        model.onHandleScan("wc:topic@2")
        await settle { model.isPresentingConnectorBar }

        #expect(model.isPresentingConnectorBar)
        #expect(model.isPresentingAlertMessage == nil)
    }

    @Test
    func aFailedPairHidesTheBarAndShowsTheError() async {
        let model = model(connector: WalletConnectorServiceMock(pairError: AnyError("bad uri")))

        model.onHandleScan("nonsense")
        await settle { model.isPresentingAlertMessage != nil }

        #expect(model.isPresentingConnectorBar == false)
        #expect(model.isPresentingAlertMessage?.message == "bad uri")
    }

    @Test
    func aFailedDisconnectShowsTheError() async {
        let model = model(connector: WalletConnectorServiceMock(disconnectError: AnyError("no session")))

        model.onSelectDisconnect(.mock())
        await settle { model.isPresentingAlertMessage != nil }

        #expect(model.isPresentingAlertMessage?.message == "no session")
    }

    @Test
    func hidingTheBarClearsIt() {
        let model = model()
        model.isPresentingConnectorBar = true

        model.hideConnectionBar()

        #expect(model.isPresentingConnectorBar == false)
    }

    private func settle(until condition: () -> Bool) async {
        for _ in 0 ..< 200 {
            await Task.yield()
            if condition() { return }
            try? await Task.sleep(for: .milliseconds(5))
        }
    }
}
