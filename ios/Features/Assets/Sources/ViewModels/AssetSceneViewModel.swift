// Copyright (c). Gem Wallet. All rights reserved.

import Components
import struct Gemstone.GemAssetBalanceRow
import enum Gemstone.GemAssetDetailRow
import struct Gemstone.GemAssetDetails
import struct Gemstone.GemAssetDetailsInput
import protocol Gemstone.GemAssetDetailsServiceProtocol
import enum Gemstone.GemBannerButton
import enum Gemstone.GemBannerDestination
import struct Gemstone.GemBannerKey
import struct Gemstone.GemFormattedNumber
import enum Gemstone.GemHeaderButtonAction
import enum Gemstone.GemInfoTopic
import enum Gemstone.GemListRowTitle
import enum Gemstone.GemLoadState
import enum Gemstone.GemServiceError
import struct Gemstone.GemTransactionRow
import func Gemstone.loadError
import GemstonePrimitives
import GemstoneServices
import Localization
import Primitives
import PrimitivesComponents
import Store
import Style
import SwiftUI
import UIKit

@Observable
@MainActor
public final class AssetSceneViewModel: Sendable {
    private let service: any GemAssetDetailsServiceProtocol
    private let preferences: ObservablePreferences
    private let onSelectPerpetuals: VoidAction

    private var isPresentingSelectedAssetInput: Binding<SelectedAssetInput?>

    public var isPresentingToastMessage: ToastMessage?
    public var isPresentingAssetSheet: AssetSheetType?

    private var transactionsState: GemLoadState = .loading

    public var input: AssetSceneInput
    public let assetQuery: ObservableQuery<ChainAssetQuery>
    public let bannersQuery: ObservableQuery<BannersQuery>
    public let transactionsQuery: ObservableQuery<MappedQuery<TransactionsQuery, [ListSection<GemTransactionRow>]>>

    public init(
        service: any GemAssetDetailsServiceProtocol,
        preferences: ObservablePreferences,
        input: AssetSceneInput,
        isPresentingSelectedAssetInput: Binding<SelectedAssetInput?>,
        onSelectPerpetuals: VoidAction = nil,
    ) {
        self.service = service
        self.preferences = preferences
        self.onSelectPerpetuals = onSelectPerpetuals

        self.input = input
        assetQuery = ObservableQuery(
            input.assetRequest,
            initialValue: ChainAssetData(
                assetData: AssetData.with(asset: input.asset),
                feeAssetData: AssetData.with(asset: input.asset.chain.asset),
            ),
        )
        bannersQuery = ObservableQuery(input.bannersRequest, initialValue: [])
        transactionsQuery = ObservableQuery(input.transactionsRequest, initialValue: [])
        self.isPresentingSelectedAssetInput = isPresentingSelectedAssetInput
    }

    public var chainAssetData: ChainAssetData {
        assetQuery.value
    }

    public var banners: [Banner] {
        bannersQuery.value
    }

    public var transactionSections: [ListSection<GemTransactionRow>] {
        transactionsQuery.value
    }

    public var assetData: AssetData {
        chainAssetData.assetData
    }

    var asset: Asset {
        assetData.asset
    }

    public var wallet: Wallet {
        input.wallet
    }

    func accessibilityIdentifier(_ row: GemAssetDetailRow) -> String? {
        switch row {
        case let .balance(item, _):
            switch item.row {
            case .staked: "stake"
            case .earn: "earn"
            case .available, .pendingUnconfirmed, .reserved: nil
            }
        case .row(.quote, _): "price"
        case .row: nil
        }
    }

    func balanceListItem(for item: GemAssetBalanceRow) -> ListItemModel {
        switch item.row {
        case .available, .staked, .earn, .reserved: ListItemModel(title: item.row.title().text, subtitle: item.value.text)
        case .pendingUnconfirmed: ListItemModel(title: item.row.title().text, subtitle: item.value.text, infoAction: onSelectPendingUnconfirmedInfo)
        }
    }

    public var details: GemAssetDetails {
        service.details(
            input: GemAssetDetailsInput(
                wallet: wallet.toGem(),
                assetData: assetData.toGem(),
                currency: preferences.currency.toGem(),
                banners: banners.map { $0.toGem() },
                feeBalanceMetadata: chainAssetData.feeAssetData.balance.metadata?.toGem(),
            ),
        )
    }

    var transactionsError: Error? {
        Gemstone.loadError(state: transactionsState, hasRows: !transactionSections.isEmpty)
    }

    var showTransactions: Bool {
        transactionSections.isNotEmpty
    }

    func emptyContentModel(_ details: GemAssetDetails) -> EmptyStateViewModel {
        EmptyStateViewModel(state: details.state.emptyState, symbol: asset.symbol) { [weak self] action in
            switch action {
            case .buy: self?.onSelectBuy()
            case .swap: self?.onSelectSwap()
            case .receive, .addCustomToken, .manageTokenList, .clearFilters: break
            }
        }
    }

    func assetHeader(_ details: GemAssetDetails) -> ValueHeader {
        details.header.valueHeader
    }

    public func shareAssetUrl(_ details: GemAssetDetails) -> URL {
        details.shareUrl.asURL!
    }

    public var optionsImage: Image {
        Images.System.ellipsis
    }

    public func priceAlertsImage(_ details: GemAssetDetails) -> Image {
        details.state.priceAlert.image
    }

    public func menuItems(_ details: GemAssetDetails) -> [ActionMenuItemType] {
        let links = details
        return [links.addressLink.map { link in
            .button(title: Localized.Asset.viewAddressOn(link.name), systemImage: SystemImage.globe, action: { self.onSelect(url: link.link.asURL) })
        },
        links.tokenLink.map { link in
            .button(title: Localized.Asset.viewTokenOn(link.name), systemImage: SystemImage.globe, action: { self.onSelect(url: link.link.asURL) })
        },
        .button(title: Localized.Common.share, systemImage: SystemImage.share, action: onSelectShareAsset)].compactMap(\.self)
    }

    func verificationStatus(_ details: GemAssetDetails) -> VerificationStatus? {
        details.verificationStatus?.toPrimitives()
    }

    var swapAssetType: SelectedAssetType {
        .swap(AssetId(core: details.swapPair.payAssetId), details.swapPair.receiveAssetId.map { AssetId(core: $0) })
    }
}

// MARK: - Business Logic

public extension AssetSceneViewModel {
    internal func loadOnce() {
        Task {
            await refresh()
        }
    }

    internal func onSelectHeader(_ action: GemHeaderButtonAction) {
        switch action {
        case .buy: onSelectBuy()
        case .send: onSelect(assetType: .send(.asset(asset: assetData.asset.toGem())))
        case let .swap(payAssetId?, receiveAssetId): onSelect(assetType: .swap(AssetId(core: payAssetId), receiveAssetId.map { AssetId(core: $0) }))
        case .receive: onSelectReceive()
        case .swap, .deposit, .withdraw, .sendCollectible, .collectibleMenu: break
        }
    }

    internal func onSelect(assetType: SelectedAssetType) {
        isPresentingSelectedAssetInput.wrappedValue = SelectedAssetInput(
            type: assetType,
            assetData: assetData,
        )
    }

    internal func onSelectWalletHeaderInfo() {
        isPresentingAssetSheet = .info(GemInfoTopic.watchWallet.infoSheet)
    }

    internal func onSelectBanner(destination: GemBannerDestination) {
        switch destination {
        case .stake:
            onSelectStake()
        case let .activateAsset(transfer):
            isPresentingAssetSheet = .transfer(transfer)
        case .perpetuals:
            preferences.isPerpetualEnabled = true
            onSelectPerpetuals?()
        case let .url(url):
            onSelect(url: URL(string: url))
        }
    }

    internal func onSelectBanner(button: GemBannerButton) {
        switch button {
        case .buy: onSelectBuy()
        case .receive: onSelectReceive()
        }
    }

    internal func onCloseBanner(_ key: GemBannerKey) {
        Task {
            do {
                try await service.closeBanner(key: key)
            } catch let error as GemServiceError {
                isPresentingToastMessage = .error(error.text().text)
            } catch {
                isPresentingToastMessage = .error(Localized.Errors.errorOccurred)
            }
        }
    }

    internal func onSelectEarn() {
        onSelect(assetType: .earn(assetData.asset))
    }

    internal func onSelectStake() {
        onSelect(assetType: .stake(assetData.asset))
    }

    private func onSelectBuy() {
        onSelect(assetType: .buy(assetData.asset, amount: nil))
    }

    private func onSelectReceive() {
        onSelect(assetType: .receive(.asset))
    }

    private func onSelectSwap() {
        onSelect(assetType: swapAssetType)
    }

    func onSelectShareAsset() {
        isPresentingAssetSheet = .share
    }

    func onTransferComplete() {
        isPresentingAssetSheet = .none
    }

    func onTogglePriceAlert() {
        Task {
            let toggled = details.state.priceAlert.toggled()
            do {
                try await setPriceAlert(enabled: toggled == .enabled)
                isPresentingToastMessage = .priceAlert(for: assetData.asset.name, enabled: toggled == .enabled)
            } catch let error as GemServiceError {
                isPresentingToastMessage = .error(error.text().text)
            } catch {
                debugLog("asset scene: price alert error \(error)")
            }
        }
    }

    func onSelectTokenStatus() {
        guard let status = verificationStatus(details) else { return }
        isPresentingAssetSheet = .info(GemInfoTopic.assetStatus(status: status.toGem()).infoSheet)
    }

    func onSelectPendingUnconfirmedInfo() {
        isPresentingAssetSheet = .info(GemInfoTopic.pendingUnconfirmedBalance.infoSheet)
    }

    func onSelect(_ title: GemListRowTitle) {
        switch title {
        case .pin, .unpin: onSelectPin()
        case .addToWallet: onSelectEnable()
        default: break
        }
    }

    func onSelectPin() {
        let pinned = !assetData.metadata.isPinned
        Task {
            do {
                try await service.setAssetPinned(assetId: asset.id.identifier, pinned: pinned)
                isPresentingToastMessage = .pin(asset.name, pinned: pinned)
            } catch {
                debugLog("onSelectPin error: \(error)")
            }
        }
    }

    func onSelectEnable() {
        Task {
            let enabled = !assetData.metadata.isBalanceEnabled
            do {
                try await service.setAssetsEnabled(assetIds: [asset.id.identifier], enabled: enabled)
                isPresentingToastMessage = .showAsset(visible: enabled)
            } catch let error as GemServiceError {
                isPresentingToastMessage = .error(error.text().text)
            } catch {
                debugLog("onSelectEnable error: \(error)")
            }
        }
    }
}

// MARK: - Private

extension AssetSceneViewModel {
    private func onSelect(url: URL?) {
        guard let url else { return }
        isPresentingAssetSheet = .url(url)
    }

    private func setPriceAlert(enabled: Bool) async throws {
        try await service.setPriceAlert(assetId: asset.id.identifier, enabled: enabled)
    }

    func refresh() async {
        let refresh = await service.refresh(assetId: asset.id.identifier, hasTransactions: showTransactions)
        transactionsState = refresh.transactions
        for failure in refresh.failures {
            debugLog("asset scene: refresh \(failure.step) failed: \(failure.message)")
        }
    }
}
