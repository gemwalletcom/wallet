// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.GemHeaderButtonKind
import BigInt
import Components
import struct Gemstone.GemAssetBalance
import protocol Gemstone.GemAssetDetailsServiceProtocol
import struct Gemstone.GemAssetDetails
import struct Gemstone.GemAssetDetailsInput
import enum Gemstone.GemBalanceRow
import struct Gemstone.GemBannerContext
import func Gemstone.assetBannerContext
import typealias Gemstone.GemBigUint
import GemstonePrimitives
import GemstoneServices
import Localization
import Primitives
import PrimitivesComponents
import Store
import Style
import SwiftUI
import UIKit
import enum Gemstone.GemServiceError

@Observable
@MainActor
public final class AssetSceneViewModel: Sendable {
    private let service: any GemAssetDetailsServiceProtocol
    private let preferences: ObservablePreferences

    private var isPresentingSelectedAssetInput: Binding<SelectedAssetInput?>

    public var isPresentingToastMessage: ToastMessage?
    public var isPresentingAssetSheet: AssetSheetType?

    public var input: AssetSceneInput
    public let assetQuery: ObservableQuery<ChainAssetRequest>
    public let bannersQuery: ObservableQuery<BannersRequest>
    public let transactionsQuery: ObservableQuery<MappedRequest<TransactionsRequest, [ListSection<TransactionViewModel>]>>

    public init(
        service: any GemAssetDetailsServiceProtocol,
        preferences: ObservablePreferences,
        input: AssetSceneInput,
        isPresentingSelectedAssetInput: Binding<SelectedAssetInput?>,
    ) {
        self.service = service
        self.preferences = preferences

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

    public var transactionSections: [ListSection<TransactionViewModel>] {
        transactionsQuery.value
    }

    public var assetData: AssetData {
        chainAssetData.assetData
    }

    private var asset: Asset {
        assetData.asset
    }

    public var wallet: Wallet {
        input.wallet
    }

    var pinListItem: ListItemModel {
        ListItemModel(title: pinText, imageStyle: .list(assetImage: AssetImage(placeholder: pinImage)))
    }

    var enableListItem: ListItemModel {
        ListItemModel(title: enableText, imageStyle: .list(assetImage: AssetImage(placeholder: enableImage)))
    }

    func priceAlertsListItem(_ details: GemAssetDetails) -> ListItemModel {
        ListItemModel(title: Localized.Settings.PriceAlerts.title, subtitle: details.state.priceAlertsCountText)
    }

    func balanceListItem(for row: GemBalanceRow) -> ListItemModel {
        switch row {
        case let .available(value): ListItemModel(title: row.title().text, subtitle: balanceText(value))
        case let .staked(value): ListItemModel(title: row.title().text, subtitle: stakeBalanceText(value))
        case let .earn(value): ListItemModel(title: row.title().text, subtitle: balanceText(value))
        case let .pendingUnconfirmed(value): ListItemModel(title: row.title().text, subtitle: balanceText(value), infoAction: onSelectPendingUnconfirmedInfo)
        case let .reserved(value, _): ListItemModel(title: row.title().text, subtitle: balanceText(value))
        }
    }

    var earnListItem: ListItemModel {
        let apr = aprModel(for: .earn)
        return ListItemModel(title: StakeProviderType.earn.title, subtitle: apr.text, subtitleStyle: apr.subtitle.style)
    }

    var balancesTitle: String {
        Localized.Asset.balances
    }

    var networkField: ListItemField {
        ListItemField(title: Localized.Transfer.network, value: assetModel.networkFullName)
    }

    var resourcesTitle: String {
        Localized.Asset.resources
    }

    var energyField: ListItemField {
        ListItemField(title: Resource.energy.title, value: feeAssetDataModel.energyText)
    }

    var bandwidthField: ListItemField {
        ListItemField(title: Resource.bandwidth.title, value: feeAssetDataModel.bandwidthText)
    }

    var balanceRows: [GemBalanceRow] {
        stakeBalance.detailRows(chain: asset.chain.rawValue, isStakeEnabled: assetData.metadata.isStakeEnabled)
    }

    public var details: GemAssetDetails {
        service.details(
            input: GemAssetDetailsInput(
                walletType: wallet.type.toGem(),
                asset: asset.toGem(),
                ownerAddress: assetDataModel.address,
                metadata: assetData.metadata.toGem(),
                balance: stakeBalance,
                price: assetData.price?.price,
                bannerEvents: visibleBanners.map { $0.event.toGem() },
                priceAlerts: assetData.priceAlerts.map { $0.toGem() },
            ),
        )
    }

    var showTransactions: Bool {
        transactionSections.isNotEmpty
    }

    var pinText: String {
        assetData.metadata.isPinned ? Localized.Common.unpin : Localized.Common.pin
    }

    var pinSystemImage: String {
        assetData.metadata.isPinned ? SystemImage.unpin : SystemImage.pin
    }

    var pinImage: Image {
        Image(systemName: pinSystemImage)
    }

    var enableText: String {
        assetData.metadata.isBalanceEnabled ? Localized.Asset.hideFromWallet : Localized.Asset.addToWallet
    }

    var enableImage: Image {
        Image(systemName: enableSystemImage)
    }

    var enableSystemImage: String {
        assetData.metadata.isBalanceEnabled ? SystemImage.minusCircle : SystemImage.plusCircle
    }

    var priceItemViewModel: PriceListItemViewModel {
        PriceListItemViewModel(
            title: Localized.Asset.price,
            model: assetDataModel.priceViewModel,
        )
    }

    var networkAssetImage: AssetImage {
        AssetIdViewModel(assetId: assetModel.asset.chain.assetId).networkAssetImage
    }

    func emptyContentModel(_ details: GemAssetDetails) -> EmptyContentTypeViewModel {
        let state = details.state
        let buy: (() -> Void)? = state.emptyTransactionsAction == .buy ? { self.onSelectBuy() } : nil
        let swap: (() -> Void)? = state.emptyTransactionsAction == .swap ? { self.onSelectSwap() } : nil
        return EmptyContentTypeViewModel(
            type: .asset(symbol: assetModel.symbol, buy: buy, swap: swap, isViewOnly: state.isViewOnly),
        )
    }

    var assetDataModel: AssetDataViewModel {
        AssetDataViewModel(
            assetData: assetData,
            formatter: .auto,
            currency: preferences.currency,
        )
    }

    var visibleBanners: [Banner] {
        bannerContext.visibleBanners(stored: banners.map { $0.toGem() }).map { $0.toPrimitives() }
    }

    func bannerModel(for banner: Banner) -> BannerViewModel {
        BannerViewModel(banner: banner, content: service.bannerContent(event: banner.event.toGem(), asset: banner.asset?.toGem()))
    }

    private var bannerContext: GemBannerContext {
        assetBannerContext(wallet: wallet.toGem(), asset: asset.toGem(), metadata: assetData.metadata.toGem(), balance: stakeBalance)
    }

    func assetHeaderModel(_ details: GemAssetDetails) -> AssetHeaderViewModel {
        AssetHeaderViewModel(assetDataModel: assetDataModel, state: details.state)
    }

    public func shareAssetUrl(_ details: GemAssetDetails) -> URL {
        details.shareUrl.asURL!
    }

    public var assetModel: AssetViewModel {
        AssetViewModel(asset: assetData.asset)
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

    func statusViewModel(_ details: GemAssetDetails) -> VerificationStatusViewModel? {
        details.verificationStatus.map { VerificationStatusViewModel(status: $0.toPrimitives()) }
    }

    var swapAssetType: SelectedAssetType {
        guard details.swapPair.receiveAssetId != nil else { return .swap(assetData.asset, nil) }
        return .swap(assetData.asset.chain.asset, assetData.asset)
    }

    func balanceText(_ value: GemBigUint) -> String {
        assetDataModel.balanceTextWithSymbol(BigInt(value))
    }

    func stakeBalanceText(_ value: GemBigUint) -> String {
        value == GemBigUint(BigInt.zero.description) ? aprModel(for: .stake).text : balanceText(value)
    }

    func aprModel(for type: StakeProviderType) -> AprViewModel {
        AprViewModel(apr: assetDataModel.apr(for: type) ?? .zero)
    }
}

// MARK: - Business Logic

public extension AssetSceneViewModel {
    internal func loadOnce() {
        Task {
            await load()
        }
    }

    internal func load() async {
        await withTaskGroup(of: Void.self) { group in
            group.addTask { await self.refresh() }
            if assetData.priceAlerts.isNotEmpty {
                group.addTask { await self.updatePriceAlerts() }
            }
        }
    }

    internal func onSelectHeader(_ buttonType: GemHeaderButtonKind) {
        let selectType: SelectedAssetType? = switch buttonType {
        case .buy: .buy(assetData.asset, amount: nil)
        case .send: .send(.asset(asset: assetData.asset.toGem()))
        case .swap: swapAssetType
        case .receive: .receive(.asset)
        case .deposit, .withdraw, .more: nil
        }
        guard let selectType else { return }
        onSelect(assetType: selectType)
    }

    internal func onSelect(assetType: SelectedAssetType) {
        isPresentingSelectedAssetInput.wrappedValue = SelectedAssetInput(
            type: assetType,
            assetData: assetData,
        )
    }

    internal func onSelectWalletHeaderInfo() {
        isPresentingAssetSheet = .info(.watchWallet)
    }

    internal func onSelectBanner(_ action: BannerAction) {
        switch action.type {
        case let .destination(destination):
            switch destination {
            case .stake:
                onSelectStake()
            case let .activateAsset(transfer):
                isPresentingAssetSheet = .transfer(transfer)
            case .perpetuals:
                UIApplication.shared.open(service.deeplinkGemUrl(deeplink: .perpetuals).asURL!)
                preferences.isPerpetualEnabled = true
            case let .url(link):
                onSelect(url: link.url)
            }
        case let .button(bannerButton):
            switch bannerButton {
            case .buy: onSelectHeader(.buy)
            case .receive: onSelectHeader(.receive)
            }
        case .closeBanner:
            Task {
                do {
                    try await service.closeBanner(key: action.banner.gemKey)
                } catch {
                    isPresentingToastMessage = .error(Localized.Errors.errorOccurred)
                }
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
        onSelectHeader(.buy)
    }

    private func onSelectSwap() {
        onSelectHeader(.swap)
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
        guard let status = statusViewModel(details)?.status else { return }
        isPresentingAssetSheet = .info(.assetStatus(status))
    }

    func onSelectPendingUnconfirmedInfo() {
        isPresentingAssetSheet = .info(.pendingUnconfirmedBalance)
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
            } catch {
                debugLog("onSelectEnable error: \(error)")
            }
        }
    }
}

// MARK: - Private

extension AssetSceneViewModel {
    private var stakeBalance: GemAssetBalance {
        GemAssetBalance(assetData.balance, assetId: asset.id, isActive: assetData.metadata.isActive)
    }

    private var feeAssetDataModel: AssetDataViewModel {
        AssetDataViewModel(
            assetData: chainAssetData.feeAssetData,
            formatter: .auto,
            currency: preferences.currency,
        )
    }

    private func onSelect(url: URL?) {
        guard let url else { return }
        isPresentingAssetSheet = .url(url)
    }

    private func setPriceAlert(enabled: Bool) async throws {
        try await service.setPriceAlert(assetId: assetModel.asset.id.identifier, enabled: enabled)
    }

    private func refresh() async {
        let failures = await service.refresh(assetId: assetModel.asset.id.identifier)
        for failure in failures {
            debugLog("asset scene: refresh \(failure.step) failed: \(failure.message)")
        }
    }

    private func updatePriceAlerts() async {
        do {
            try await service.syncPriceAlerts(assetId: asset.id.identifier)
        } catch {
            debugLog("asset scene: price alerts update error \(error)")
        }
    }
}
