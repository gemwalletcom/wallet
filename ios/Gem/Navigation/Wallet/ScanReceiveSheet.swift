// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Primitives
import SwiftUI

extension View {
    func scanReceiveSheet(isPresented: Binding<Bool>, action: StringAction) -> some View {
        modifier(ScanReceiveSheet(isPresented: isPresented, action: action))
    }
}

private struct ScanReceiveSheet: ViewModifier {
    @Environment(\.viewModelFactory) private var viewModelFactory

    @Binding var isPresented: Bool

    @State private var code: String?

    let action: StringAction

    func body(content: Content) -> some View {
        content.sheet(isPresented: $isPresented, onDismiss: onDismiss) {
            if let selectAssetModel = viewModelFactory.selectAssetScene(selectType: .receive(.asset)) {
                ScanReceiveNavigationStack(
                    model: ScanReceiveViewModel(
                        selectAssetModel: selectAssetModel,
                        onScan: { code = $0 },
                    ),
                )
                .orientationLock(.portrait)
            }
        }
    }

    private func onDismiss() {
        guard let code else { return }
        self.code = .none
        action?(code)
    }
}
