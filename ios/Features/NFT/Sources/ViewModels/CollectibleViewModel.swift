// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Formatters
import Foundation
import enum Gemstone.GemCollectibleAction
import struct Gemstone.GemCollectibleAttribute
import enum Gemstone.GemCollectibleAttributeValue
import struct Gemstone.GemCollectibleDetails
import protocol Gemstone.GemCollectibleServiceProtocol
import enum Gemstone.GemHeaderButtonKind
import struct Gemstone.GemInfoSheet
import enum Gemstone.GemInfoTopic
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
    private let gallery: any ImageGallerySaving
    private let onSelectAddress: (@MainActor @Sendable (ChainAddress) -> Void)?

    public let query: ObservableQuery<NFTAssetQuery>

    var isPresentingAlertMessage: AlertMessage?
    var isPresentingToast: ToastMessage?
    var isPresentingSelectedAssetInput: Binding<SelectedAssetInput?>
    var isPresentingReportSheet = false
    var isPresentingInfoSheet: GemInfoSheet?
    var isImageLoaded = false

    public init(
        wallet: Wallet,
        assetData: NFTAssetData,
        service: any GemCollectibleServiceProtocol,
        gallery: any ImageGallerySaving = ImageGalleryService(),
        isPresentingSelectedAssetInput: Binding<SelectedAssetInput?>,
        onSelectAddress: (@MainActor @Sendable (ChainAddress) -> Void)? = nil,
    ) {
        self.wallet = wallet
        self.onSelectAddress = onSelectAddress
        self.service = service
        self.gallery = gallery
        self.isPresentingSelectedAssetInput = isPresentingSelectedAssetInput
        query = ObservableQuery(
            NFTAssetQuery(walletId: wallet.id, assetId: assetData.asset.id),
            initialValue: NFTAssetDetails(assetData: assetData, isOwned: false),
        )
    }

    var assetData: NFTAssetData {
        query.value.assetData
    }

    var title: String {
        assetData.asset.name
    }

    func imageContextMenuItems(_ details: GemCollectibleDetails) -> [ContextMenuItemType] {
        guard isImageLoaded else { return [] }
        return details.imageActions.compactMap(contextMenuItem)
    }

    var details: GemCollectibleDetails {
        service.details(walletType: wallet.type.toGem(), assetData: assetData.toGem(), isOwned: query.value.isOwned, canSaveImage: true)
    }

    var assetImage: AssetImage {
        AssetImage(type: .text(assetData.asset.name), imageURL: assetData.asset.images.preview.url.asURL, placeholder: .none, chainPlaceholder: .none)
    }

    func headerButtons(_ details: GemCollectibleDetails) -> [HeaderButton] {
        details.header.headerButtons.map { button in
            guard button.type == .more else { return button }
            return HeaderButton(type: .more, viewType: .menuButton(title: title, items: details.actions.map(menuItem)), isEnabled: button.isEnabled)
        }
    }

    private func contextMenuItem(_ action: GemCollectibleAction) -> ContextMenuItemType? {
        switch action {
        case .saveImage: .custom(title: Localized.Nft.saveToPhotos, systemImage: SystemImage.gallery, action: onSelectSaveToGallery)
        case .setAvatar: .custom(title: Localized.Nft.setAsAvatar, systemImage: SystemImage.emoji, action: onSelectSetAsAvatar)
        case .refresh, .report: nil
        }
    }

    private func menuItem(_ action: GemCollectibleAction) -> ActionMenuItemType {
        switch action {
        case .saveImage: .button(title: Localized.Nft.saveToPhotos, systemImage: SystemImage.gallery, action: onSelectSaveToGallery)
        case .setAvatar: .button(title: Localized.Nft.setAsAvatar, systemImage: SystemImage.emoji, action: onSelectSetAsAvatar)
        case .refresh: .button(title: Localized.Common.refresh, systemImage: SystemImage.refresh, action: onSelectRefresh)
        case .report: .button(title: Localized.Nft.Report.reportButtonTitle, role: .destructive, action: onSelectReport)
        }
    }

    func attributeListItem(_ attribute: GemCollectibleAttribute) -> ListItemModel {
        ListItemModel(title: attribute.name, subtitle: attributeText(attribute.value))
    }

    func attributeText(_ value: GemCollectibleAttributeValue) -> String {
        switch value {
        case let .text(value): value
        case let .date(date): TransactionDateFormatter(date: date).day
        }
    }
}

// MARK: - Business Logic

extension CollectibleViewModel {
    func onSelectHeaderButton(type: GemHeaderButtonKind) {
        guard let account = try? wallet.account(for: assetData.asset.chain) else {
            return
        }
        switch type {
        case .send:
            isPresentingSelectedAssetInput.wrappedValue = SelectedAssetInput(
                type: .send(.nft(nftAsset: assetData.asset.toGem())),
                assetData: .with(asset: account.chain.asset, account: account),
            )
        case .buy, .receive, .swap, .more, .deposit, .withdraw:
            break
        }
    }

    func onSelectSaveToGallery() {
        Task {
            await saveToGallery()
        }
    }

    func saveToGallery() async {
        do throws(ImageGalleryServiceError) {
            try await saveImageToGallery()
            isPresentingToast = .success(Localized.Nft.saveToPhotos)
        } catch {
            switch error {
            case .wrongURL, .invalidData, .invalidResponse, .unexpectedStatusCode, .urlSessionError, .saveFailed:
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

    func onSelectSetAsAvatar() {
        Task {
            do {
                try await setWalletAvatar()
                isPresentingToast = .success(Localized.Nft.setAsAvatar)
            } catch {
                isPresentingAlertMessage = AlertMessage(error: error)
            }
        }
    }

    var onSelectContract: ((String) -> Void)? {
        guard let onSelectAddress else { return nil }
        let chain = assetData.asset.chain
        return { onSelectAddress(ChainAddress(chain: chain, address: $0)) }
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
        isPresentingInfoSheet = GemInfoTopic.assetStatus(status: assetData.collection.status.toGem()).infoSheet
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
        try await gallery.saveImageFromURL(url)
    }
}
