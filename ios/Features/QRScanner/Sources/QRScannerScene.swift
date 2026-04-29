// Copyright (c). Gem Wallet. All rights reserved.

import Components
import PhotosUI
import SwiftUI

struct QRScannerScene: View {
    @Environment(\.openURL) private var openURL
    @Environment(\.dismiss) private var dismiss

    @State private var model: QRScannerSceneViewModel

    private let action: (String) -> Void

    init(resources: QRScannerResources, action: @escaping ((String) -> Void)) {
        self.action = action
        _model = State(initialValue: QRScannerSceneViewModel(scannerState: .idle, imageState: .empty, resources: resources))
    }

    var body: some View {
        ZStack {
            switch model.scannerState {
            case .idle, .scanning:
                QRScannerDisplayView(
                    configuration: model.overlayConfig,
                    isScannerReady: $model.isScannerReady,
                    scanResult: onHandleScanResult,
                )
                .ignoresSafeArea()
            case let .failure(error):
                ContentUnavailableView(
                    label: {
                        if let titleImage = error.titleImage {
                            Label(titleImage.title, systemImage: titleImage.systemImage)
                        }
                    },
                    description: {
                        Text(error.localizedDescription)
                    },
                    actions: {
                        switch error {
                        case .notSupported:
                            let text = model.resources.selectFromPhotos
                            photosPicker {
                                Text(text)
                            }
                        case .permissionsNotGranted:
                            Button(model.resources.openSettings, action: onSelectOpenSettings)
                        case .decoding, .unknown:
                            Button(model.resources.tryAgain, action: model.refreshScannerState)
                        }
                    },
                )
            }
        }
        .toolbar {
            ToolbarItem(placement: .primaryAction) {
                let imageName = model.resources.gallerySystemImage
                photosPicker {
                    Image(systemName: imageName)
                        .bold()
                }
            }
            ToolbarItem(placement: .cancellationAction) {
                Button {
                    dismiss()
                } label: {
                    Text(model.resources.dismissText)
                }
            }
        }
        .toolbarBackground(.visible, for: .navigationBar)
        .onChange(
            of: model.isScannerReady,
            initial: true,
            model.onChangeScannerReadyStatus,
        )
        .onChange(of: model.selectedPhoto, onLoadPhoto)
        .onChange(of: model.imageState, onChangeImageState)
    }

    private func photosPicker(
        @ViewBuilder label: @Sendable @escaping () -> some View,
    ) -> some View {
        PhotosPicker(
            selection: $model.selectedPhoto,
            matching: .images,
            photoLibrary: .shared(),
            label: label,
        )
    }
}

// MARK: - Actions

extension QRScannerScene {
    private func onSelectOpenSettings() {
        guard let settingsURL = URL(string: UIApplication.openSettingsURLString) else { return }
        openURL(settingsURL)
    }

    private func onLoadPhoto(_: PhotosPickerItem?, _ newValue: PhotosPickerItem?) {
        guard let newValue else { return }
        Task {
            await model.process(photoItem: newValue)
        }
    }

    private func onChangeImageState(_: ImageState, _ newValue: ImageState) {
        guard case let .success(uIImage) = newValue else { return }

        do {
            let code = try model.retrieveQRCode(image: uIImage)
            onHandleScanResult(.success(code))
        } catch {
            model.refreshScannerState(error: error)
        }
    }

    private func onHandleScanResult(_ result: Result<String, Error>) {
        guard case let .success(code) = result else { return }
        action(code)
        dismiss()
    }
}
