// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Localization
import PhotosUI
import Primitives
import PrimitivesComponents
import Style
import SwiftUI

@Observable
@MainActor
public final class QRScannerSceneViewModel {
    var scannerState: QRScannerState = .scanning
    var selectedPhoto: PhotosPickerItem?
    var isPresentingToastMessage: ToastMessage?
    var isScannerReady: Bool = false

    let scanType: QRScanType

    private let action: (String) -> Void

    public init(scanType: QRScanType, action: @escaping (String) -> Void) {
        self.scanType = scanType
        self.action = action
    }

    var selectFromPhotos: String {
        Localized.Library.selectFromPhotoLibrary
    }

    var openSettings: String {
        Localized.Common.openSettings
    }

    var gallerySystemImage: String {
        SystemImage.photo
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
        scanType.hint
    }
}

// MARK: - Business Logic

extension QRScannerSceneViewModel {
    func onChangeScannerReadyStatus(_: Bool, _: Bool) {
        refreshScannerState()
    }

    func onScenePhaseChange(_: ScenePhase, _ newPhase: ScenePhase) {
        switch newPhase {
        case .active: refreshScannerState()
        case .inactive, .background: break
        @unknown default: break
        }
    }

    func onScan(_ code: String) {
        action(code)
    }

    func scanSelectedPhoto() async -> Bool {
        guard let selectedPhoto else { return false }
        let code = await retrieveQRCode(photoItem: selectedPhoto)
        guard !Task.isCancelled else { return false }
        self.selectedPhoto = nil
        guard let code else {
            showDecodingError()
            return false
        }
        onScan(code)
        return true
    }
}

// MARK: - Private

extension QRScannerSceneViewModel {
    private func retrieveQRCode(photoItem: PhotosPickerItem) async -> String? {
        guard let data = try? await photoItem.loadTransferable(type: Data.self),
              let image = UIImage(data: data, scale: 1.0)
        else {
            return nil
        }
        return QRImageDecoder.decode(image)
    }

    private func showDecodingError() {
        UINotificationFeedbackGenerator().notificationOccurred(.error)
        isPresentingToastMessage = ToastMessage(title: Localized.Errors.decodingQr, image: SystemImage.xmarkCircle)
    }

    private func refreshScannerState() {
        #if targetEnvironment(macCatalyst)
            scannerState = .failure(error: .notSupported)
        #else
            do {
                try QRScannerViewWrapper.checkDeviceQRScanningSupport()
                scannerState = .scanning
            } catch {
                scannerState = .failure(error: error)
            }
        #endif
    }
}
