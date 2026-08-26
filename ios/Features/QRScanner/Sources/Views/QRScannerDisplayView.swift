// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Style
import SwiftUI

struct QRScannerDisplayView: View {
    private let configuration: QRScannerDisplayConfiguration
    private let hint: String
    private let scanResult: QRScannerViewWrapper.ScanResult
    @Binding private var isScannerReady: Bool

    init(
        configuration: QRScannerDisplayConfiguration,
        hint: String,
        isScannerReady: Binding<Bool>,
        scanResult: @escaping QRScannerViewWrapper.ScanResult,
    ) {
        _isScannerReady = isScannerReady
        self.configuration = configuration
        self.hint = hint
        self.scanResult = scanResult
    }

    private var cornerRadius: CGFloat {
        configuration.cornerRadius
    }

    var body: some View {
        GeometryReader { geometry in
            let boxSize = min(geometry.size.width, geometry.size.height) * configuration.squareScale
            let captionTopInset = (geometry.size.height + boxSize) / 2 + configuration.captionSpacing

            ZStack {
                configuration.overlayColor

                QRScannerViewWrapper(
                    isScannerReady: $isScannerReady,
                    scanResult: scanResult,
                )

                configuration.overlayColor
                    .opacity(configuration.dimmedViewOpacity)
                    .overlay {
                        RoundedRectangle(cornerRadius: cornerRadius)
                            .fill(configuration.overlayColor)
                            .frame(width: boxSize, height: boxSize)
                            .blendMode(.destinationOut)
                    }
                    .compositingGroup()

                CornerBracketsView(
                    configuration: configuration,
                    boxSize: boxSize,
                    geometrySize: geometry.size,
                )

                caption
                    .padding(.top, captionTopInset)
                    .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .top)
                    .allowsHitTesting(false)
            }
        }
    }

    private var caption: some View {
        Text(hint)
            .font(.footnote)
            .foregroundStyle(configuration.captionColor)
            .multilineTextAlignment(.center)
            .padding(.horizontal, .medium)
            .padding(.vertical, .small)
            .liquidGlass(interactive: false, fallback: {
                $0.background(.ultraThinMaterial, in: Capsule())
            })
            .padding(.horizontal, .medium)
    }
}

#Preview {
    ZStack {
        Color.red
            .ignoresSafeArea()
        QRScannerDisplayView(
            configuration: .default,
            hint: "Send crypto or connect to an app",
            isScannerReady: .constant(true),
            scanResult: { _ in },
        )
    }
}
