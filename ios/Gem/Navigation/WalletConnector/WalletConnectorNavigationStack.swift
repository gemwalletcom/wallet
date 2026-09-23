// Copyright (c). Gem Wallet. All rights reserved.

import GemstoneServices
import Primitives
import Style
import SwiftUI
import Transfer
import WalletConnector

struct WalletConnectorNavigationStack: View {
    @Environment(\.viewModelFactory) private var viewModelFactory

    private let type: WalletConnectorSheetType
    private let presenter: WalletConnectorPresenter

    init(
        type: WalletConnectorSheetType,
        presenter: WalletConnectorPresenter,
    ) {
        self.type = type
        self.presenter = presenter
    }

    var body: some View {
        NavigationStack {
            Group {
                switch type {
                case let .transferData(data):
                    ConfirmTransferNavigationView(
                        model: viewModelFactory.confirmTransferScene(
                            wallet: data.payload.wallet,
                            data: data.payload.transferData,
                            confirmTransferDelegate: data.delegate,
                            simulation: data.payload.simulation,
                            onComplete: { presenter.complete(type: type) },
                        ),
                    )
                case let .signMessage(data):
                    SignMessageNavigationView(
                        model: viewModelFactory.signMessageScene(
                            request: data.payload,
                            confirmTransferDelegate: data.delegate,
                        ),
                        onComplete: { presenter.complete(type: type) },
                    )
                case let .connectionProposal(data):
                    ConnectionProposalScene(
                        model: ConnectionProposalViewModel(
                            confirmTransferDelegate: data.delegate,
                            pairingProposal: data.payload,
                        ),
                        onComplete: { presenter.complete(type: type) },
                    )
                }
            }
            .interactiveDismissDisabled(true)
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .topBarLeading) {
                    Button("", systemImage: SystemImage.xmark) {
                        presenter.cancelSheet(type: type)
                    }
                }
            }
        }
    }
}

private struct SignMessageNavigationView: View {
    @State private var model: SignMessageSceneViewModel
    private let onComplete: () -> Void

    init(
        model: SignMessageSceneViewModel,
        onComplete: @escaping () -> Void,
    ) {
        _model = State(initialValue: model)
        self.onComplete = onComplete
    }

    var body: some View {
        SignMessageScene(model: model, onComplete: onComplete)
            .sheet(item: $model.presentedSheet) { sheet in
                switch sheet {
                case .payloadDetails:
                    SignMessagePayloadDetails(model: model)
                case let .addressDetails(chainAddress):
                    AddressDetailsDestination(chainAddress: chainAddress)
                }
            }
    }
}
