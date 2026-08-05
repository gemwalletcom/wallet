// Copyright (c). Gem Wallet. All rights reserved.

import SwiftUI
import WalletConnector

struct WalletConnectorNavigationStack: View {
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
                    SigningRequestSheetView(content: .transfer(data), onComplete: complete)
                case let .signMessage(data):
                    SigningRequestSheetView(content: .signMessage(data), onComplete: complete)
                case let .connectionProposal(data):
                    ConnectionProposalScene(
                        model: ConnectionProposalViewModel(
                            confirmTransferDelegate: data.delegate,
                            pairingProposal: data.payload,
                        ),
                        onComplete: complete,
                    )
                }
            }
            .signingRequestChrome(onCancel: { presenter.cancelSheet(type: type) })
        }
    }

    private func complete() {
        presenter.complete(type: type)
    }
}
