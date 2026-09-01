// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Localization
import Style

public struct QRScanResources: QRScannerResources {
    public init() {}

    public var selectFromPhotos: String {
        Localized.Library.selectFromPhotoLibrary
    }

    public var openSettings: String {
        Localized.Common.openSettings
    }

    public var tryAgain: String {
        Localized.Common.tryAgain
    }

    public var dismissText: String {
        Localized.Common.cancel
    }

    public var gallerySystemImage: String {
        SystemImage.photo
    }
}
