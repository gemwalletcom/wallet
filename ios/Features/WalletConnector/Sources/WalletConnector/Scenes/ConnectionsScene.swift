// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Primitives
import PrimitivesComponents
import QRScanner
import Store
import Style
import SwiftUI

public struct ConnectionsScene: View {
    @State private var model: ConnectionsSceneViewModel

    public init(model: ConnectionsSceneViewModel) {
        _model = State(initialValue: model)
    }

    public var body: some View {
        let view = model.view
        return List {
            Section {
                ButtonListItem(
                    title: model.pasteButtonTitle,
                    image: Images.System.paste,
                    action: onPaste,
                )
                ButtonListItem(
                    title: model.scanQRCodeButtonTitle,
                    image: Images.System.qrCodeViewfinder,
                    action: model.onScan,
                )
            }

            GemGroupSections(groups: view.list.groups) { item in
                if case let .connection(row) = item {
                    let connection = row.connection.toPrimitives()
                    NavigationLink(value: connection) {
                        ConnectionView(connection: row)
                            .swipeActions(edge: .trailing) {
                                Button(
                                    model.disconnectTitle,
                                    role: .destructive,
                                    action: { onSelectDisconnect(connection) },
                                )
                                .tint(Colors.red)
                            }
                    }
                }
            }
        }
        .bindQuery(model.query)
        .contentMargins(.top, .scene.top, for: .scrollContent)
        .overlay {
            if case let .empty(state) = view.list.phase {
                EmptyContentView(model: EmptyStateViewModel(state: state))
                    .padding(.horizontal, .medium)
            }
        }
        .navigationDestination(keyedBy: WalletConnection.self) { connection in
            ConnectionScene(
                details: model.connectionDetails(connection: connection),
                onDisconnect: { onSelectDisconnect(connection) },
            )
        }
        .sheet(isPresented: $model.isPresentingScanner) {
            QRScannerNavigationStack(scanType: .walletConnect, action: onHandleScan)
        }
        .ifLet(view.docsUrl.asURL) { content, url in
            content.toolbarInfoButton(url: url)
        }
        .alertSheet($model.isPresentingAlertMessage)
        .navigationTitle(model.title)
        .taskOnce(onLoad)
    }
}

// MARK: - Actions

extension ConnectionsScene {
    private func onLoad() {
        Task { await model.load() }
    }

    private func onPaste() {
        Task { await model.onPaste() }
    }

    private func onHandleScan(_ result: String) {
        Task { await model.onHandleScan(result) }
    }

    private func onSelectDisconnect(_ connection: WalletConnection) {
        Task { await model.onSelectDisconnect(connection) }
    }
}
