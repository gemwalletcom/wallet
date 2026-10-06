// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import enum Gemstone.GemCollectibleAction
import struct Gemstone.GemCollectibleAttribute
import enum Gemstone.GemCollectibleAttributeValue
import struct Gemstone.GemCollectibleDetails
import protocol Gemstone.GemCollectibleServiceProtocol
import enum Gemstone.GemHeaderButtonAction
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
public final class CollectibleSceneViewModel {
    private let wallet: Wallet
    private let assetId: NFTAssetId
    private let service: any GemCollectibleServiceProtocol
    private let gallery: any ImageGallerySaving
    private let onSelectAddress: (@MainActor @Sendable (ChainAddress) -> Void)?

    public let query: ObservableQuery<NFTAssetQuery>

    private var loadState: StateViewType<NFTAssetDetails> = .loading

    var isPresentingAlertMessage: AlertMessage?
    var isPresentingToast: ToastMessage?
    var isPresentingSelectedAssetInput: Binding<SelectedAssetInput?>
    var isPresentingReportSheet = false
    var isPresentingInfoSheet: GemInfoSheet?
    var isImageLoaded = false

    public init(
        wallet: Wallet,
        collectible: Scenes.Collectible,
        service: any GemCollectibleServiceProtocol,
        gallery: any ImageGallerySaving = ImageGalleryService(),
        isPresentingSelectedAssetInput: Binding<SelectedAssetInput?>,
        onSelectAddress: (@MainActor @Sendable (ChainAddress) -> Void)? = nil,
    ) {
        self.wallet = wallet
        assetId = collectible.assetId
        self.onSelectAddress = onSelectAddress
        self.service = service
        self.gallery = gallery
        self.isPresentingSelectedAssetInput = isPresentingSelectedAssetInput
        let stored: NFTAssetDetails? = switch collectible {
        case let .assetData(assetData): NFTAssetDetails(assetData: assetData, isOwned: false)
        case .assetId: nil
        }
        query = ObservableQuery(NFTAssetQuery(walletId: wallet.id, assetId: collectible.assetId), initialValue: stored)
    }

    var state: StateViewType<NFTAssetDetails> {
        query.value.map { .data($0) } ?? loadState
    }

    var assetData: NFTAssetData? {
        query.value?.assetData
    }

    var title: String {
        switch state {
        case .loading, .noData: Localized.Common.loading
        case .error: Localized.Errors.error
        case let .data(details): details.assetData.asset.name
        }
    }

    var errorTitle: String {
        Localized.Errors.errorOccurred
    }

    func imageContextMenuItems(_ details: GemCollectibleDetails) -> [ContextMenuItemType] {
        guard isImageLoaded else { return [] }
        return details.imageActions.map { .custom(title: $0.title, systemImage: $0.systemImage, action: onSelect($0)) }
    }

    var details: GemCollectibleDetails? {
        query.value.map(details)
    }

    func details(_ details: NFTAssetDetails) -> GemCollectibleDetails {
        service.details(walletType: wallet.type.toGem(), assetData: details.assetData.toGem(), isOwned: details.isOwned, canSaveImage: true)
    }

    func assetImage(_ assetData: NFTAssetData) -> AssetImage {
        AssetImage(type: .text(assetData.asset.name), imageURL: assetData.asset.images.preview.url.asURL, placeholder: .none, chainPlaceholder: .none)
    }

    func menuItems(_ details: GemCollectibleDetails) -> [ActionMenuItemType] {
        details.actions.map { row in
            .button(title: row.action.title, systemImage: row.action.systemImage, role: row.isDestructive ? .destructive : nil, action: onSelect(row.action))
        }
    }

    private func onSelect(_ action: GemCollectibleAction) -> VoidAction {
        switch action {
        case .saveImage: onSelectSaveToGallery
        case .setAvatar: onSelectSetAsAvatar
        case .refresh: onSelectRefresh
        case .report: onSelectReport
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

extension CollectibleSceneViewModel {
    func load() async {
        guard query.value == nil else { return }
        loadState = .loading
        do {
            _ = try await service.ensureAsset(assetId: assetId.identifier)
        } catch {
            loadState = .error(error)
        }
    }

    func onSelectHeaderButton(_ action: GemHeaderButtonAction) {
        guard let assetData, let account = try? wallet.account(for: assetData.asset.chain) else {
            return
        }
        switch action {
        case .sendCollectible:
            isPresentingSelectedAssetInput.wrappedValue = SelectedAssetInput(
                type: .send(.nft(nftAsset: assetData.asset.toGem())),
                assetData: .with(asset: account.chain.asset, account: account),
            )
        case .send, .receive, .buy, .swap, .deposit, .withdraw, .collectibleMenu:
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
        let chain = assetId.chain
        return { onSelectAddress(ChainAddress(chain: chain, address: $0)) }
    }

    func onSelectReport() {
        isPresentingReportSheet = true
    }

    func onSelectRefresh() {
        Task {
            do {
                try await service.refreshAsset(assetId: assetId.identifier)
                isPresentingToast = .success(Localized.Common.refresh)
            } catch {
                debugLog("Refresh nft asset error: \(error)")
                isPresentingAlertMessage = AlertMessage(message: Localized.Errors.errorOccurred)
            }
        }
    }

    func reportModel(_ assetData: NFTAssetData) -> ReportNftSceneViewModel {
        ReportNftSceneViewModel(service: service, assetData: assetData, onComplete: onReportComplete)
    }

    func onReportComplete() {
        isPresentingReportSheet = false
        isPresentingToast = .success(Localized.Transaction.Status.confirmed)
    }

    func onSelectStatus() {
        guard let assetData else { return }
        isPresentingInfoSheet = GemInfoTopic.assetStatus(status: assetData.collection.status.toGem()).infoSheet
    }
}

// MARK: - Private

extension CollectibleSceneViewModel {
    private func openSettings() {
        guard let settingsURL = URL(string: UIApplication.openSettingsURLString) else { return }
        UIApplication.shared.open(settingsURL)
    }

    private func setWalletAvatar() async throws {
        guard let url = assetData?.asset.images.preview.url.asURL else { return }
        try await service.setWalletAvatar(url: url.absoluteString)
    }

    private func saveImageToGallery() async throws(ImageGalleryServiceError) {
        guard let url = assetData?.asset.images.preview.url.asURL else {
            throw ImageGalleryServiceError.wrongURL
        }
        try await gallery.saveImageFromURL(url)
    }
}
