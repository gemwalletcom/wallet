// Copyright (c). Gem Wallet. All rights reserved.

import ExplorerService
import Primitives
import Signer
import Style
import SwiftUI
import TransactionStateService
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
                            data: data.payload.tranferData,
                            confirmTransferDelegate: data.delegate,
                            simulation: data.payload.simulation,
                            onComplete: { presenter.complete(type: type) },
                        ),
                    )
                case let .signMessage(data):
                    SignMessageScene(
                        model: viewModelFactory.signMessageScene(
                            payload: data.payload,
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
