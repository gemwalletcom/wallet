// Copyright (c). Gem Wallet. All rights reserved.

import SigningRequestService
import Style
import SwiftUI
import Transfer
import WalletConnector

struct SigningRequestSheetView: View {
    @Environment(\.viewModelFactory) private var viewModelFactory

    private let content: SigningRequestSheetContent
    private let onComplete: () -> Void

    init(
        content: SigningRequestSheetContent,
        onComplete: @escaping () -> Void,
    ) {
        self.content = content
        self.onComplete = onComplete
    }

    var body: some View {
        switch content {
        case let .transfer(data):
            ConfirmTransferNavigationView(
                model: viewModelFactory.confirmTransferScene(
                    wallet: data.payload.wallet,
                    data: data.payload.transferData,
                    confirmTransferDelegate: data.delegate,
                    simulation: data.payload.simulation,
                    onComplete: onComplete,
                ),
            )
        case let .signMessage(data):
            SignMessageScene(
                model: viewModelFactory.signMessageScene(
                    payload: data.payload,
                    confirmTransferDelegate: data.delegate,
                ),
                onComplete: onComplete,
            )
        }
    }
}

enum SigningRequestSheetContent {
    case transfer(SigningRequestCallback<SigningTransferData>)
    case signMessage(SigningRequestCallback<SignMessagePayload>)
}

// MARK: - Chrome

extension View {
    func signingRequestSheet(onCancel: @escaping () -> Void) -> some View {
        NavigationStack {
            self
                .interactiveDismissDisabled(true)
                .navigationBarTitleDisplayMode(.inline)
                .toolbar {
                    ToolbarItem(placement: .topBarLeading) {
                        Button("", systemImage: SystemImage.xmark, action: onCancel)
                    }
                }
        }
    }
}
