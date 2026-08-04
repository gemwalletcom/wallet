// Copyright (c). Gem Wallet. All rights reserved.

import Style
import SwiftUI
import Primitives

public struct ConnectionProposalNavigationStack: View {
    private let type: WalletConnectorSheetType
    private let presenter: SheetPresenter<WalletConnectorSheetType>

    public init(
        type: WalletConnectorSheetType,
        presenter: SheetPresenter<WalletConnectorSheetType>,
    ) {
        self.type = type
        self.presenter = presenter
    }

    public var body: some View {
        NavigationStack {
            Group {
                switch type {
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
