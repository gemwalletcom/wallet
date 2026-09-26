// Copyright (c). Gem Wallet. All rights reserved.

import Components
import struct Gemstone.GemConnection
import Localization
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
        let sections = view.sections
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

            ForEach(Array(sections.enumerated()), id: \.offset) { _, section in
                Section(section.title) {
                    ForEach(section.connections, id: \.connection.session.id) { item in
                        let connection = item.connection.toPrimitives()
                        NavigationLink(value: connection) {
                            ConnectionView(model: ConnectionViewModel(connection: item))
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
        }
        .bindQuery(model.query)
        .contentMargins(.top, .scene.top, for: .scrollContent)
        .overlay {
            if sections.isEmpty {
                EmptyContentView(model: model.emptyContentModel)
                    .padding(.horizontal, .medium)
            }
        }
        .navigationDestination(for: WalletConnection.self) { connection in
            ConnectionScene(
                model: model.connectionSceneModel(connection: connection),
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
        .toast(
            isPresenting: $model.isPresentingConnectorBar,
            message: ToastMessage(
                title: "\(Localized.WalletConnect.brandName)...",
                image: SystemImage.network,
            ),
            duration: .infinity,
            tapToDismiss: false,
        )
        .navigationTitle(model.title)
        .taskOnce { model.load() }
        .onChange(of: model.walletConnectorPresenter?.isPresentingSheet?.id, model.hideConnectionBar)
    }
}

// MARK: - Actions

extension ConnectionsScene {
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
