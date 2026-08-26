// Copyright (c). Gem Wallet. All rights reserved.

import SwiftUI
import VisionKit

@MainActor
final class ScannerHostViewController: UIViewController {
    private let scanner: DataScannerViewController

    var onDidAppear: (() -> Void)?

    init(scanner: DataScannerViewController) {
        self.scanner = scanner
        super.init(nibName: nil, bundle: nil)
    }

    @available(*, unavailable)
    required init?(coder _: NSCoder) {
        fatalError()
    }

    override func viewDidLoad() {
        super.viewDidLoad()
        addChild(scanner)
        scanner.view.frame = view.bounds
        scanner.view.autoresizingMask = [.flexibleWidth, .flexibleHeight]
        view.addSubview(scanner.view)
        scanner.didMove(toParent: self)
    }

    override func viewDidAppear(_ animated: Bool) {
        super.viewDidAppear(animated)
        onDidAppear?()
    }
}

@MainActor
struct QRScannerViewWrapper {
    typealias ScanResult = (Result<String, Error>) -> Void

    private var scanResult: ScanResult
    private var dataScannerVC = DataScannerViewController(
        recognizedDataTypes: [.barcode(symbologies: [.qr])],
        qualityLevel: .balanced,
        recognizesMultipleItems: false,
        isHighFrameRateTrackingEnabled: true,
        isGuidanceEnabled: false,
        isHighlightingEnabled: false,
    )

    @Binding var isScannerReady: Bool

    init(isScannerReady: Binding<Bool>, scanResult: @escaping ScanResult) {
        self.scanResult = scanResult
        _isScannerReady = isScannerReady
    }

    func startScanning() {
        guard !dataScannerVC.isScanning else { return }
        do {
            try dataScannerVC.startScanning()
            isScannerReady = true
        } catch {
            scanResult(.failure(QRScannerError.unknown(error)))
            isScannerReady = false
        }
    }

    static func checkDeviceQRScanningSupport() throws {
        guard DataScannerViewController.isSupported else {
            throw QRScannerError.notSupported
        }

        guard DataScannerViewController.isAvailable else {
            throw QRScannerError.permissionsNotGranted
        }
    }
}

// MARK: - UIViewControllerRepresentable

extension QRScannerViewWrapper: UIViewControllerRepresentable {
    func makeUIViewController(context: Context) -> ScannerHostViewController {
        dataScannerVC.delegate = context.coordinator
        let host = ScannerHostViewController(scanner: dataScannerVC)
        host.onDidAppear = startScanning
        return host
    }

    func updateUIViewController(_: ScannerHostViewController, context _: Context) {}

    func makeCoordinator() -> Coordinator {
        Coordinator(self)
    }
}

// MARK: - Coordinator

extension QRScannerViewWrapper {
    @MainActor
    class Coordinator {
        var parent: QRScannerViewWrapper

        init(_ parent: QRScannerViewWrapper) {
            self.parent = parent
        }

        func didAddItem(item: RecognizedItem) {
            guard case let .barcode(barcode) = item else { return }

            if let code = barcode.payloadStringValue {
                parent.scanResult(.success(code))
            } else {
                parent.scanResult(.failure(QRScannerError.decoding))
            }
        }
    }
}

// MARK: - DataScannerViewControllerDelegate

extension QRScannerViewWrapper.Coordinator: DataScannerViewControllerDelegate {
    func dataScanner(_: DataScannerViewController, didAdd addedItems: [RecognizedItem], allItems _: [RecognizedItem]) {
        if let item = addedItems.first {
            didAddItem(item: item)
        }
    }

    func dataScanner(
        _: DataScannerViewController,
        becameUnavailableWithError error: DataScannerViewController.ScanningUnavailable,
    ) {
        parent.scanResult(.failure(error))
    }
}
