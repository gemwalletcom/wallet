// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Localization
import PhotosUI
import Primitives
import Style
import SwiftUI

@Observable
@MainActor
final class QRScannerSceneViewModel {
    var scannerState: QRScannerState = .scanning
    var selectedPhoto: PhotosPickerItem?
    var isPresentingToastMessage: ToastMessage?
    var isScannerReady: Bool = false

    let resources: QRScannerResources
    let scanType: QRScanType

    init(resources: QRScannerResources, scanType: QRScanType) {
        self.resources = resources
        self.scanType = scanType
    }

    var overlayConfig: QRScannerDisplayConfiguration {
        .default
    }

    var toastOffset: CGFloat {
        switch scannerState {
        case .scanning: .zero
        case .failure(.notSupported): .scene.button.height + .medium
        case .failure(.permissionsNotGranted): .scene.button.height * 2 + .medium * 2
        }
    }

    var hint: String {
        switch scanType {
        case .universal: Localized.Wallet.scanHint
        case .walletConnect: Localized.WalletConnect.title
        case .address: Localized.Wallet.scanHintAddress
        case .memo: Localized.Transfer.memo
        case .url: Localized.Common.url
        case .tokenContract: Localized.Wallet.Import.contractAddressField
        case .secretPhrase: Localized.Common.secretPhrase
        case .privateKey: Localized.Common.privateKey
        }
    }
}

// MARK: - Business Logic

extension QRScannerSceneViewModel {
    func onChangeScannerReadyStatus(_: Bool, _: Bool) {
        refreshScannerState()
    }

    func retrieveQRCode(photoItem: PhotosPickerItem) async -> String? {
        guard let data = try? await photoItem.loadTransferable(type: Data.self),
              let image = UIImage(data: data, scale: 1.0)
        else {
            return nil
        }
        return QRImageDecoder.decode(image)
    }

    func showDecodingError() {
        UINotificationFeedbackGenerator().notificationOccurred(.error)
        isPresentingToastMessage = ToastMessage(title: Localized.Errors.decodingQr, image: SystemImage.xmarkCircle)
    }
}

// MARK: - Private

extension QRScannerSceneViewModel {
    private func refreshScannerState() {
        do {
            try QRScannerViewWrapper.checkDeviceQRScanningSupport()
            scannerState = .scanning
        } catch {
            scannerState = .failure(error: error)
        }
    }
}
