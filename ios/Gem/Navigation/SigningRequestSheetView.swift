// Copyright (c). Gem Wallet. All rights reserved.

import Primitives
import PrimitivesComponents
import Style
import SwiftUI
import Transfer

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
    case transfer(SheetCallback<SigningTransferData>)
    case signMessage(SheetCallback<SignMessagePayload>)
}

// MARK: - Chrome

extension View {
    func signingRequestChrome(onCancel: @escaping () -> Void) -> some View {
        interactiveDismissDisabled(true)
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .topBarLeading) {
                    Button("", systemImage: SystemImage.xmark, action: onCancel)
                }
            }
    }
}
