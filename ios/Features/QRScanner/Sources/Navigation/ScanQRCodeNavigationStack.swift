// Copyright (c). Gem Wallet. All rights reserved.

import Localization
import Style
import SwiftUI

public struct ScanQRCodeNavigationStack: View {
    @Environment(\.dismiss) private var dismiss

    private let resources = QRScanResources()

    let action: (String) -> Void

    public init(action: @escaping (String) -> Void) {
        self.action = action
    }

    public var body: some View {
        NavigationStack {
            QRScannerScene(resources: resources, action: action)
                .navigationTitle(Localized.Wallet.scanQrCode)
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

// MARK: - LocalizedQRCodeError

extension QRScannerError: LocalizedQRCodeError {
    public var titleImage: ErrorTitleImage? {
        switch self {
        case .notSupported:
            ErrorTitleImage(title: Localized.Errors.notSupported, systemImage: SystemImage.clear)
        case .permissionsNotGranted:
            ErrorTitleImage(title: Localized.Errors.permissionsNotGranted, systemImage: SystemImage.lock)
        case .decoding:
            ErrorTitleImage(title: Localized.Errors.decoding, systemImage: SystemImage.errorOccurred)
        case .unknown:
            ErrorTitleImage(title: Localized.Errors.unknown, systemImage: SystemImage.errorOccurred)
        }
    }

    public var errorDescription: String? {
        switch self {
        case .notSupported:
            Localized.Errors.notSupportedQr
        case .permissionsNotGranted:
            Localized.Errors.cameraPermissionsNotGranted
        case .decoding:
            Localized.Errors.decodingQr
        case .unknown:
            Localized.Errors.unknown
        }
    }
}
