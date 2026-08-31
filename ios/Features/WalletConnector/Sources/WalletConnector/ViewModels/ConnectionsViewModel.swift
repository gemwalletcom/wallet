// Copyright (c). Gem Wallet. All rights reserved.

import class Gemstone.GemApplicationMetadataService
import Components
import WalletConnectorService
import Foundation
import GemstonePrimitives
import Localization
import Primitives
import PrimitivesComponents
import Store
import UIKit

@Observable
@MainActor
public final class ConnectionsViewModel {
    let connector: any WalletConnectorServiceable
    let walletConnectorPresenter: WalletConnectorPresenter?

    public let query: ObservableQuery<ConnectionsRequest>
    var connections: [WalletConnection] {
        query.value
    }

    var isPresentingScanner: Bool = false
    var isPresentingAlertMessage: AlertMessage?
    var isPresentingConnectorBar: Bool = false
    private let applicationMetadataService: GemApplicationMetadataService

    public init(
        connector: any WalletConnectorServiceable,
        applicationMetadataService: GemApplicationMetadataService,
        walletConnectorPresenter: WalletConnectorPresenter? = nil,
    ) {
        self.applicationMetadataService = applicationMetadataService
        self.connector = connector
        self.walletConnectorPresenter = walletConnectorPresenter
        query = ObservableQuery(ConnectionsRequest(), initialValue: [])
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

    var docsUrl: URL {
        AppUrl.docs(.walletConnect)
    }

    var sections: [ListSection<WalletConnection>] {
        let grouped = Dictionary(grouping: connections, by: { $0.wallet })
        return grouped.keys
            .sorted { $0.index < $1.index }
            .map { wallet in
                ListSection(
                    id: wallet.id.id,
                    title: wallet.name,
                    image: nil,
                    values: grouped[wallet]?.sorted { $0.session.createdAt > $1.session.createdAt } ?? [],
                )
            }
    }

    var emptyContentModel: EmptyContentTypeViewModel {
        EmptyContentTypeViewModel(type: .walletConnect)
    }

    func connectionViewModel(connection: WalletConnection) -> WalletConnectionViewModel {
        WalletConnectionViewModel(connection: connection, applicationMetadataService: applicationMetadataService)
    }

    func connectionSceneModel(connection: WalletConnection) -> ConnectionSceneViewModel {
        ConnectionSceneViewModel(
            model: connectionViewModel(connection: connection),
            connector: connector,
        )
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

    func onPaste() {
        guard let content = UIPasteboard.general.string else {
            return
        }

        Task {
            await connectURI(uri: content)
        }
    }

    func onHandleScan(_ result: String) {
        Task {
            await connectURI(uri: result)
        }
    }

    func onSelectDisconnect(_ connection: WalletConnection) {
        Task {
            do {
                try await disconnect(connection: connection)
            } catch {
                isPresentingAlertMessage = AlertMessage(message: error.localizedDescription)
                debugLog("disconnect error: \(error)")
            }
        }
    }

    private func connectURI(uri: String) async {
        isPresentingConnectorBar = true
        do {
            try await pair(uri: uri)
        } catch {
            hideConnectionBar()
            isPresentingAlertMessage = AlertMessage(message: error.localizedDescription)
            debugLog("connectURI error: \(error)")
        }
    }
}
