// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import class Gemstone.GemAddressService
import struct Gemstone.GemCollectibleLinks
import protocol Gemstone.GemCollectibleServiceProtocol
import GemstonePrimitives
import GemstoneServices
import ImageGalleryService
import InfoSheet
import Localization
import Primitives
import PrimitivesComponents
import Style
import SwiftUI

@Observable
@MainActor
public final class CollectibleViewModel {
    private let wallet: Wallet
    private let service: any GemCollectibleServiceProtocol

    let assetData: NFTAssetData

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
        self.assetData = assetData
        self.service = service
        self.isPresentingSelectedAssetInput = isPresentingSelectedAssetInput
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

    var collectionField: ListItemField {
        ListItemField(title: Localized.Nft.collection, value: assetData.collection.name)
    }

    var isVerified: Bool {
        assetData.collection.status == .verified
    }

    var networkField: ListItemField {
        ListItemField(title: Localized.Transfer.network, value: assetData.asset.chain.networkName)
    }

    var contractField: ListItemField? {
        if contractValue.isEmpty || contractValue == assetData.asset.tokenId {
            return .none
        }
        let text = GemAddressService.shared.format(address: contractValue, chain: assetData.asset.chain)
        return ListItemField(title: Localized.Asset.contract, value: text)
    }

    var contractExplorerLink: BlockExplorerLink? {
        links.contract.map { $0.map() }
    }

    var contractExplorerContext: ExplorerContextData? {
        contractExplorerLink.map {
            ExplorerContextData(copyValue: .address(value: contractValue, chain: assetData.asset.chain), explorerLink: $0)
        }
    }

    var contractRow: CollectibleInfoRow? {
        contractField.map {
            CollectibleInfoRow(
                field: $0,
                action: contractExplorerContext.map { .explorer($0) } ?? .copy(contractValue),
            )
        }
    }

    var tokenIdValue: String {
        assetData.asset.tokenId
    }

    var tokenIdField: ListItemField {
        let text = if assetData.asset.tokenId.count > 16 {
            GemAddressService.shared.format(address: assetData.asset.tokenId, chain: assetData.asset.chain)
        } else {
            "#\(assetData.asset.tokenId)"
        }
        return ListItemField(title: Localized.Asset.tokenId, value: text)
    }

    var tokenIdExplorerLink: BlockExplorerLink? {
        links.token.map { $0.map() }
    }

    var tokenIdExplorerContext: ExplorerContextData? {
        tokenIdExplorerLink.map {
            ExplorerContextData(copyValue: .plain(tokenIdValue), explorerLink: $0)
        }
    }

    var attributesTitle: String {
        Localized.Nft.properties
    }

    var attributes: [NFTAttributeViewModel] {
        assetData.asset.attributes.map { NFTAttributeViewModel(attribute: $0) }
    }

    var assetImage: AssetImage {
        NFTAssetViewModel(asset: assetData.asset).assetImage
    }

    var networkAssetImage: AssetImage {
        AssetImage(
            imageURL: .none,
            placeholder: ChainImage(chain: assetData.asset.chain).image,
            chainPlaceholder: .none,
        )
    }

    var isSendEnabled: Bool {
        service.canSend(wallet: wallet.json(), chain: assetData.asset.chain.map())
    }

    var headerButtons: [HeaderButton] {
        [
            HeaderButton(
                type: .send,
                isEnabled: isSendEnabled,
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

    var showAttributes: Bool {
        attributes.isNotEmpty
    }

    var showLinks: Bool {
        assetData.collection.links.isNotEmpty
    }

    var statusViewModel: VerificationStatusViewModel {
        VerificationStatusViewModel(status: assetData.collection.status)
    }

    var showStatus: Bool {
        assetData.collection.status != .verified
    }

    var socialLinksViewModel: SocialLinksViewModel {
        SocialLinksViewModel(assetLinks: assetData.collection.links)
    }

    var tokenIdRow: CollectibleInfoRow {
        CollectibleInfoRow(
            field: tokenIdField,
            action: tokenIdExplorerContext.map { .explorer($0) } ?? .copy(tokenIdValue),
        )
    }
}

// MARK: - Business Logic

extension CollectibleViewModel {
    func onSelectCopyValue(_ value: CopyValue) {
        isPresentingToast = .copied(value.displayValue)
    }

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
                type: .send(.nft(assetData.asset)),
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
    private var contractValue: String {
        assetData.collection.contractAddress
    }

    private var links: GemCollectibleLinks {
        service.links(chain: assetData.asset.chain.rawValue, contractAddress: contractValue, tokenId: tokenIdValue)
    }

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
