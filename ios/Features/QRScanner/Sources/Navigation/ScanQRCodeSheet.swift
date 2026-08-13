// Copyright (c). Gem Wallet. All rights reserved.

import SwiftUI

public extension View {
    func scanQRCodeSheet(isPresented: Binding<Bool>, action: @escaping (String) -> Void) -> some View {
        modifier(ScanQRCodeSheet(isPresented: isPresented, action: action))
    }
}

private struct ScanQRCodeSheet: ViewModifier {
    @Binding var isPresented: Bool

    @State private var scanned: String?

    let action: (String) -> Void

    func body(content: Content) -> some View {
        content.sheet(isPresented: $isPresented, onDismiss: onDismiss) {
            ScanQRCodeNavigationStack { scanned = $0 }
        }
    }

    private func onDismiss() {
        guard let scanned else { return }
        self.scanned = .none
        action(scanned)
    }
}
