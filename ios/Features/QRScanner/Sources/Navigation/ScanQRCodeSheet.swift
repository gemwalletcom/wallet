// Copyright (c). Gem Wallet. All rights reserved.

import Primitives
import SwiftUI

public extension View {
    func scanQRCodeSheet(isPresented: Binding<Bool>, scanType: QRScanType, action: @escaping (String) -> Void) -> some View {
        modifier(ScanQRCodeSheet(isPresented: isPresented, scanType: scanType, action: action))
    }
}

private struct ScanQRCodeSheet: ViewModifier {
    @Binding var isPresented: Bool

    @State private var code: String?

    let scanType: QRScanType
    let action: (String) -> Void

    func body(content: Content) -> some View {
        content.sheet(isPresented: $isPresented, onDismiss: onDismiss) {
            ScanQRCodeNavigationStack(scanType: scanType) { code = $0 }
        }
    }

    private func onDismiss() {
        guard let code else { return }
        self.code = .none
        action(code)
    }
}
