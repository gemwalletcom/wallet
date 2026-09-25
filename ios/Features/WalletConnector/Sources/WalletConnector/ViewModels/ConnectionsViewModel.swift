// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import struct Gemstone.GemConnection
import struct Gemstone.GemConnectionsView
import protocol Gemstone.GemWalletConnectServiceProtocol
import func Gemstone.walletConnectErrorText
import GemstonePrimitives
import Localization
import Primitives
import PrimitivesComponents
import Store
import UIKit
import WalletConnectorService

@Observable
@MainActor
public final class ConnectionsViewModel {
    let connector: any WalletConnectorServiceable
    let walletConnectorPresenter: WalletConnectorPresenter?
    private let service: any GemWalletConnectServiceProtocol

    public let query: ObservableQuery<ConnectionsQuery>
    var connections: [WalletConnection] {
        query.value
    }

    var isPresentingScanner: Bool = false
    var isPresentingAlertMessage: AlertMessage?
    var isPresentingConnectorBar: Bool = false

    public init(
        connector: any WalletConnectorServiceable,
        service: any GemWalletConnectServiceProtocol,
        walletConnectorPresenter: WalletConnectorPresenter? = nil,
    ) {
        self.connector = connector
        self.service = service
        self.walletConnectorPresenter = walletConnectorPresenter
        query = ObservableQuery(ConnectionsQuery(), initialValue: [])
    }

    var title: String {
        Localized.WalletConnect.title
    }

    var disconnectTitle: String {
        Localized.WalletConnect.disconnect
    }

    var pasteButtonTitle: String {
        Localized.Common.paste
    }

    var scanQRCodeButtonTitle: String {
        Localized.Wallet.scanQrCode
    }

    var view: GemConnectionsView {
        service.connectionsView(connections: connections.map { $0.toGem() })
    }

    var emptyContentModel: EmptyContentTypeViewModel {
        EmptyContentTypeViewModel(type: EmptyContentType(.walletConnect))
    }

    func connectionSceneModel(connection: WalletConnection) -> ConnectionSceneViewModel {
        ConnectionSceneViewModel(details: service.connectionDetails(connection: connection.toGem()))
    }

    func pair(uri: String) async throws {
        try await connector.pair(uri: uri)
    }

    func disconnect(connection: WalletConnection) async throws {
        try await connector.disconnect(sessionId: connection.session.sessionId)
    }

    func load() {
        connector.updateSessions()
    }

    func hideConnectionBar() {
        isPresentingConnectorBar = false
    }
}

// MARK: - Actions

extension ConnectionsViewModel {
    func onScan() {
        isPresentingScanner = true
    }

    func onPaste() async {
        guard let content = UIPasteboard.general.string else {
            return
        }
        await connectURI(uri: content)
    }

    func onHandleScan(_ result: String) async {
        await connectURI(uri: result)
    }

    func onSelectDisconnect(_ connection: WalletConnection) async {
        do {
            try await disconnect(connection: connection)
        } catch {
            isPresentingAlertMessage = AlertMessage(message: walletConnectErrorText(message: error.localizedDescription).text)
            debugLog("disconnect error: \(error)")
        }
    }

    private func connectURI(uri: String) async {
        isPresentingConnectorBar = true
        do {
            try await pair(uri: uri)
        } catch {
            hideConnectionBar()
            isPresentingAlertMessage = AlertMessage(message: walletConnectErrorText(message: error.localizedDescription).text)
            debugLog("connectURI error: \(error)")
        }
    }
}
