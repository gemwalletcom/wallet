// Copyright (c). Gem Wallet. All rights reserved.

import SwiftUI

enum QRImageDecoder {
    static func decode(_ image: UIImage) -> String? {
        guard let ciImage = CIImage(image: image) else {
            return nil
        }
        return detectQRCode(in: ciImage)
    }

    private static func detectQRCode(in image: CIImage) -> String? {
        let detector = CIDetector(
            ofType: CIDetectorTypeQRCode,
            context: nil,
            options: [CIDetectorAccuracy: CIDetectorAccuracyHigh],
        )
        let features = detector?.features(in: image) as? [CIQRCodeFeature]
        return features?.first?.messageString
    }
}
