// Copyright (c). Gem Wallet. All rights reserved.

import Primitives
import SwiftUI

extension View {
    func scanReceiveSheet(isPresented: Binding<Bool>, action: StringAction) -> some View {
        modifier(ScanReceiveSheet(isPresented: isPresented, action: action))
    }
}

private struct ScanReceiveSheet: ViewModifier {
    @Environment(\.viewModelFactory) private var viewModelFactory
    @Environment(\.walletSessionService) private var walletSessionService

    @Binding var isPresented: Bool

    @State private var code: String?

    let action: StringAction

    func body(content: Content) -> some View {
        content.sheet(isPresented: $isPresented, onDismiss: onDismiss) {
            if let wallet = walletSessionService.currentWallet {
                ScanReceiveNavigationStack(
                    model: ScanReceiveViewModel(
                        selectAssetModel: viewModelFactory.selectAssetScene(
                            wallet: wallet,
                            selectType: .receive(.asset),
                        ),
                        onScan: { code = $0 },
                    ),
                )
            }
        }
    }

    private func onDismiss() {
        guard let code else { return }
        self.code = .none
        action?(code)
    }
}
