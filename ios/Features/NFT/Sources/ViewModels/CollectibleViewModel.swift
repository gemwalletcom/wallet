// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Formatters
import Foundation
import enum Gemstone.GemCollectibleAttributeValue
import struct Gemstone.GemCollectibleDetails
import enum Gemstone.GemCollectibleSection
import protocol Gemstone.GemCollectibleServiceProtocol
import GemstonePrimitives
import GemstoneServices
import ImageGalleryService
import InfoSheet
import Localization
import Primitives
import PrimitivesComponents
import Store
import Style
import SwiftUI

@Observable
@MainActor
public final class CollectibleViewModel {
    private let wallet: Wallet
    private let service: any GemCollectibleServiceProtocol
    private let dateFormatter = RelativeDateFormatter(type: .date)

    public let query: ObservableQuery<NFTAssetRequest>

    var isPresentingAlertMessage: AlertMessage?
    var isPresentingToast: ToastMessage?
    var isPresentingSelectedAssetInput: Binding<SelectedAssetInput?>
    var isPresentingReportSheet = false
    var isPresentingInfoSheet: InfoSheetType?
    var isImageLoaded = false

    public init(
        wallet: Wallet,
        assetData: NFTAssetData,
        service: any GemCollectibleServiceProtocol,
        isPresentingSelectedAssetInput: Binding<SelectedAssetInput?>,
    ) {
        self.wallet = wallet
        self.service = service
        self.isPresentingSelectedAssetInput = isPresentingSelectedAssetInput
        query = ObservableQuery(
            NFTAssetRequest(walletId: wallet.id, assetId: assetData.asset.id),
            initialValue: NFTAssetDetails(assetData: assetData, isOwned: false),
        )
    }

    var assetData: NFTAssetData {
        query.value.assetData
    }

    var title: String {
        assetData.asset.name
    }

    var imageContextMenuItems: [ContextMenuItemType] {
        guard isImageLoaded else { return [] }
        return [
            .custom(
                title: Localized.Nft.saveToPhotos,
                systemImage: SystemImage.gallery,
                action: onSelectSaveToGallery,
            ),
            .custom(
                title: Localized.Nft.setAsAvatar,
                systemImage: SystemImage.emoji,
                action: onSelectSetAsAvatar,
            ),
        ]
    }

    var isVerified: Bool {
        assetData.collection.status == .verified
    }

    var details: GemCollectibleDetails {
        service.details(walletType: wallet.type.map(), assetData: assetData.map(), isOwned: query.value.isOwned)
    }

    var sections: [GemCollectibleSection] {
        details.sections
    }

    var assetImage: AssetImage {
        NFTAssetViewModel(asset: assetData.asset).assetImage
    }

    var headerButtons: [HeaderButton] {
        [
            HeaderButton(
                type: .send,
                isEnabled: details.canSend,
            ),
            HeaderButton(
                type: .more,
                viewType: .menuButton(
                    title: title,
                    items: [
                        .button(title: Localized.Nft.saveToPhotos, systemImage: SystemImage.gallery, action: onSelectSaveToGallery),
                        .button(title: Localized.Nft.setAsAvatar, systemImage: SystemImage.emoji, action: onSelectSetAsAvatar),
                        .button(title: Localized.Common.refresh, systemImage: SystemImage.refresh, action: onSelectRefresh),
                        .button(title: Localized.Nft.Report.reportButtonTitle, role: .destructive, action: onSelectReport),
                    ],
                ),
                isEnabled: true,
            ),
        ]
    }

    func networkImage(chain: Chain) -> AssetImage {
        AssetImage(
            imageURL: .none,
            placeholder: ChainImage(chain: chain).image,
            chainPlaceholder: .none,
        )
    }

    func attributeText(_ value: GemCollectibleAttributeValue) -> String {
        switch value {
        case let .text(value): value
        case let .date(date): dateFormatter.string(from: date)
        }
    }
}

// MARK: - Business Logic

extension CollectibleViewModel {
    func onSelectCopyValue(_ value: String) {
        isPresentingToast = .copied(value)
    }

    func onSelectHeaderButton(type: HeaderButtonType) {
        guard let account = try? wallet.account(for: assetData.asset.chain) else {
            return
        }
        switch type {
        case .send:
            isPresentingSelectedAssetInput.wrappedValue = SelectedAssetInput(
                type: .send(.nft(nftAsset: assetData.asset.map())),
                assetData: .with(asset: account.chain.asset, account: account),
            )
        case .buy, .sell, .receive, .swap, .stake, .more, .deposit, .withdraw:
            fatalError()
        }
    }

    func onSelectSaveToGallery() {
        Task {
            do {
                try await saveImageToGallery()
                isPresentingToast = .success(Localized.Nft.saveToPhotos)
            } catch let error as ImageGalleryServiceError {
                switch error {
                case .wrongURL, .invalidData, .invalidResponse, .unexpectedStatusCode, .urlSessionError:
                    isPresentingAlertMessage = AlertMessage(message: Localized.Errors.errorOccurred)
                case .permissionDenied:
                    isPresentingAlertMessage = AlertMessage(
                        title: Localized.Permissions.accessDenied,
                        message: Localized.Permissions.Image.PhotoAccess.Denied.description,
                        actions: [
                            AlertAction(
                                title: Localized.Common.openSettings,
                                isDefaultAction: true,
                                action: {
                                    Task { @MainActor in
                                        self.openSettings()
                                    }
                                },
                            ),
                            .cancel(title: Localized.Common.cancel),
                        ],
                    )
                }
            }
        }
    }

    func onSelectSetAsAvatar() {
        Task {
            do {
                try await setWalletAvatar()
                isPresentingToast = .success(Localized.Nft.setAsAvatar)
            } catch {
                debugLog("Set nft avatar error: \(error)")
            }
        }
    }

    func onSelectReport() {
        isPresentingReportSheet = true
    }

    func onSelectRefresh() {
        Task {
            do {
                try await service.refreshAsset(assetId: assetData.asset.id.identifier)
                isPresentingToast = .success(Localized.Common.refresh)
            } catch {
                debugLog("Refresh nft asset error: \(error)")
                isPresentingAlertMessage = AlertMessage(message: Localized.Errors.errorOccurred)
            }
        }
    }

    func reportModel() -> ReportNftViewModel {
        ReportNftViewModel(service: service, assetData: assetData, onComplete: onReportComplete)
    }

    func onReportComplete() {
        isPresentingReportSheet = false
        isPresentingToast = .success(Localized.Transaction.Status.confirmed)
    }

    func onSelectStatus() {
        isPresentingInfoSheet = .assetStatus(assetData.collection.status)
    }
}

// MARK: - Private

extension CollectibleViewModel {
    private func openSettings() {
        guard let settingsURL = URL(string: UIApplication.openSettingsURLString) else { return }
        UIApplication.shared.open(settingsURL)
    }

    private func setWalletAvatar() async throws {
        guard let url = assetData.asset.images.preview.url.asURL else { return }
        try await service.setWalletAvatar(url: url.absoluteString)
    }

    private func saveImageToGallery() async throws(ImageGalleryServiceError) {
        guard let url = assetData.asset.images.preview.url.asURL else {
            throw ImageGalleryServiceError.wrongURL
        }
        let saver = ImageGalleryService()
        try await saver.saveImageFromURL(url)
    }
}
