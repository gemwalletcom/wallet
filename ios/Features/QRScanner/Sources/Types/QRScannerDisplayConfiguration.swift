// Copyright (c). Gem Wallet. All rights reserved.

import Style
import SwiftUI

struct QRScannerDisplayConfiguration {
    let squareScale: CGFloat

    let cornerRadius: CGFloat
    let cornerLength: CGFloat

    let lineWidth: CGFloat
    let overlayColor: Color
    let bracketColor: Color

    let dimmedViewOpacity: CGFloat

    let captionColor: Color
    let captionSpacing: CGFloat

    init(
        dimmedViewOpacity: CGFloat,
        squareScale: CGFloat,
        cornerRadius: CGFloat,
        cornerLength: CGFloat,
        lineWidth: CGFloat,
        overlayColor: Color,
        bracketColor: Color,
        captionColor: Color,
        captionSpacing: CGFloat,
    ) {
        self.dimmedViewOpacity = dimmedViewOpacity
        self.squareScale = squareScale
        self.cornerRadius = cornerRadius
        self.cornerLength = cornerLength
        self.lineWidth = lineWidth
        self.overlayColor = overlayColor
        self.bracketColor = bracketColor
        self.captionColor = captionColor
        self.captionSpacing = captionSpacing
    }
}

extension QRScannerDisplayConfiguration {
    static let `default` = QRScannerDisplayConfiguration(
        dimmedViewOpacity: 0.33,
        squareScale: 0.66,
        cornerRadius: 25,
        cornerLength: 25,
        lineWidth: 4,
        overlayColor: .black,
        bracketColor: .white,
        captionColor: .white,
        captionSpacing: .large,
    )
}
