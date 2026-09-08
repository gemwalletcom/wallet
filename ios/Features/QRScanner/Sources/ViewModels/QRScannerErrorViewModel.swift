// Copyright (c). Gem Wallet. All rights reserved.

import Localization
import Style
import SwiftUI

struct QRScannerErrorViewModel {
    let error: QRScannerError

    var title: String {
        switch error {
        case .notSupported: Localized.Errors.notSupported
        case .permissionsNotGranted: Localized.Errors.permissionsNotGranted
        }
    }

    var description: String {
        switch error {
        case .notSupported: Localized.Errors.notSupportedQr
        case .permissionsNotGranted: Localized.Errors.cameraPermissionsNotGranted
        }
    }

    var image: Image {
        switch error {
        case .notSupported: Images.System.qrCodeViewfinder
        case .permissionsNotGranted: Images.System.camera
        }
    }
}
