// Copyright (c). Gem Wallet. All rights reserved.

import Localization
import Primitives
import SwiftUI

public struct ScanQRCodeNavigationStack: View {
    @Environment(\.dismiss) private var dismiss

    private let resources = QRScanResources()
    private let scanType: QRScanType

    let action: (String) -> Void

    public init(scanType: QRScanType, action: @escaping (String) -> Void) {
        self.scanType = scanType
        self.action = action
    }

    public var body: some View {
        NavigationStack {
            QRScannerScene(resources: resources, scanType: scanType, action: action)
                .navigationTitle(Localized.Wallet.scan)
                .navigationBarTitleDisplayMode(.inline)
                .toolbar {
                    ToolbarItem(placement: .cancellationAction) {
                        Button(resources.dismissText) {
                            dismiss()
                        }
                    }
                }
        }
    }
}
