// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Components
import struct Gemstone.GemAssetBalance
import protocol Gemstone.GemAssetDetailsServiceProtocol
import enum Gemstone.GemAssetNetworkDestination
import enum Gemstone.GemBalanceRow
import struct Gemstone.GemBannerContent
import struct Gemstone.GemBannerContext
import typealias Gemstone.GemBigUint
import struct Gemstone.GemRecipient
import struct Gemstone.GemTransferData
import GemstonePrimitives
import GemstoneServices
import Localization
import Preferences
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

    private var isPresentingSelectedAssetInput: Binding<SelectedAssetInput?>

    public var isPresentingToastMessage: ToastMessage?
    public var isPresentingAssetSheet: AssetSheetType?

    public var input: AssetSceneInput
    public let assetQuery: ObservableQuery<ChainAssetRequest>
    public let bannersQuery: ObservableQuery<BannersRequest>
    public let transactionsQuery: ObservableQuery<TransactionsRequest>

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

    public var transactions: [TransactionExtended] {
        transactionsQuery.value
    }

    public var assetData: AssetData {
        chainAssetData.assetData
    }

    private var asset: Asset {
        assetData.asset
    }

    private var wallet: Wallet {
        walletModel.wallet
    }

    public var title: String {
        assetModel.name
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
        ListItemField(title: ResourceViewModel(resource: .energy).title, value: feeAssetDataModel.energyText)
    }

    var bandwidthField: ListItemField {
        ListItemField(title: ResourceViewModel(resource: .bandwidth).title, value: feeAssetDataModel.bandwidthText)
    }

    var networkDestination: GemAssetNetworkDestination? {
        service.networkDestination(assetId: asset.id.identifier)
    }

    var balanceRows: [GemBalanceRow] {
        let rows = stakeBalance.detailRows(chain: asset.chain.rawValue, isStakeEnabled: assetData.metadata.isStakeEnabled)
        #if DEBUG
            return rows
        #else
            return rows.filter { if case .earn = $0 { false } else { true } }
        #endif
    }

    var showResources: Bool {
        assetDataModel.showResources
    }

    var showTransactions: Bool {
        transactions.isNotEmpty
    }

    var showManageToken: Bool {
        !assetData.metadata.isBalanceEnabled
    }

    var canSign: Bool {
        wallet.canSign
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

    var reservedBalanceUrl: URL? {
        assetModel.asset.chain.accountActivationFeeUrl
    }

    var showEarnButton: Bool {
        #if DEBUG
            assetData.metadata.isEarnEnabled && !wallet.isViewOnly && !balanceRows.contains { if case .earn = $0 { true } else { false } }
        #else
            false
        #endif
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

    var emptyContentModel: EmptyContentTypeViewModel {
        let buy = assetData.metadata.isBuyEnabled ? onSelectBuy : nil
        let swap = buy == nil && assetData.metadata.isSwapEnabled ? onSelectSwap : nil
        return EmptyContentTypeViewModel(
            type: .asset(symbol: assetModel.symbol, buy: buy, swap: swap, isViewOnly: wallet.isViewOnly),
        )
    }

    var assetDataModel: AssetDataViewModel {
        AssetDataViewModel(
            assetData: assetData,
            formatter: .auto,
            currencyCode: preferences.currency,
        )
    }

    var visibleBanners: [Banner] {
        do {
            return try bannerContext.visibleBanners(banners, walletId: wallet.id, asset: assetData.asset)
        } catch {
            debugLog("asset scene: visible banners error \(error)")
            return []
        }
    }

    func bannerContent(for banner: Banner) -> GemBannerContent {
        service.bannerContent(event: banner.event.json(), asset: banner.asset?.map())
    }

    private var bannerContext: GemBannerContext {
        GemBannerContext(
            wallet: wallet.json(),
            hasAsset: true,
            isStakeable: assetData.metadata.isStakeEnabled,
            hasStakeBalance: stakedValue > .zero,
            hasAvailableBalance: assetData.balance.available > 0,
            isAssetActivated: assetData.metadata.isActive,
            assetRankScore: assetData.metadata.rankScore,
            isWalletEmpty: false,
        )
    }

    var assetHeaderModel: AssetHeaderViewModel {
        AssetHeaderViewModel(
            assetDataModel: assetDataModel,
            walletModel: walletModel,
            bannerEventsViewModel: HeaderBannerEventViewModel(events: visibleBanners.map(\.event)),
        )
    }

    public var shareAssetUrl: URL {
        service.deeplinkUrl(deeplink: DeepLink.asset(assetDataModel.asset.id).map()).asURL!
    }

    public var assetModel: AssetViewModel {
        AssetViewModel(asset: assetData.asset)
    }

    public var walletModel: WalletViewModel {
        WalletViewModel(wallet: input.wallet)
    }

    public var optionsImage: Image {
        Images.System.ellipsis
    }

    public var priceAlertsSystemImage: String {
        assetData.isPriceAlertsEnabled ? SystemImage.bellFill : SystemImage.bell
    }

    public var priceAlertsImage: Image {
        Image(systemName: priceAlertsSystemImage)
    }

    public var showPriceAlerts: Bool {
        priceAlertsViewModel.hasPriceAlerts && assetDataModel.isPriceAvailable
    }

    public var menuItems: [ActionMenuItemType] {
        [.button(title: viewAddressOnTitle, systemImage: SystemImage.globe, action: { self.onSelect(url: self.addressExplorerUrl) }),
         viewTokenOnTitle.map { .button(title: $0, systemImage: SystemImage.globe, action: { self.onSelect(url: self.tokenExplorerUrl) }) },
         .button(title: Localized.Common.share, systemImage: SystemImage.share, action: onSelectShareAsset)].compactMap(\.self)
    }

    var statusViewModel: VerificationStatusViewModel? {
        service.verificationStatus(asset: assetData.asset.map(), rank: assetData.metadata.rankScore).map { VerificationStatusViewModel(status: $0.map()) }
    }

    var priceAlertsViewModel: PriceAlertsViewModel {
        PriceAlertsViewModel(priceAlerts: assetData.priceAlerts)
    }

    var swapAssetType: SelectedAssetType {
        let pair = service.swapPair(assetId: assetData.asset.id.identifier, hasBalance: assetData.balance.available > .zero)
        guard pair.receiveAssetId != nil else { return .swap(assetData.asset, nil) }
        return .swap(assetData.asset.chain.asset, assetData.asset)
    }

    func balanceText(_ value: GemBigUint) -> String {
        assetDataModel.balanceTextWithSymbol(BigInt(value))
    }

    func stakeBalanceText(_ value: GemBigUint) -> String {
        value == GemBigUint(BigInt.zero.description) ? aprModel(for: .stake).text : balanceText(value)
    }

    func balanceTitle(for type: StakeProviderType) -> String {
        switch type {
        case .stake: Localized.Wallet.stake
        case .earn: Localized.Common.earn
        }
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

    internal func onSelectHeader(_ buttonType: HeaderButtonType) {
        let selectType: SelectedAssetType = switch buttonType {
        case .buy: .buy(assetData.asset, amount: nil)
        case .sell: .sell(assetData.asset, amount: nil)
        case .send: .send(.asset(assetData.asset))
        case .swap: swapAssetType
        case .receive: .receive(.asset)
        case .stake: .stake(assetData.asset)
        case .more, .deposit, .withdraw:
            fatalError()
        }
        isPresentingSelectedAssetInput.wrappedValue = SelectedAssetInput(
            type: selectType,
            assetData: assetData,
        )
    }

    internal func onSelectWalletHeaderInfo() {
        isPresentingAssetSheet = .info(.watchWallet)
    }

    internal func onSelectBanner(_ action: BannerAction) {
        switch action.type {
        case let .event(event):
            switch event {
            case .stake:
                onSelectHeader(.stake)
            case .activateAsset:
                isPresentingAssetSheet = .transfer(
                    GemTransferData(
                        inputType: .account(assetData.asset, .activate),
                        recipient: GemRecipient(address: ""),
                        value: BigInt.zero,
                    ),
                )
            case .accountActivation,
                 .accountBlockedMultiSignature,
                 .onboarding:
                Task {
                    try await service.applyBannerAction(key: action.banner.gemKey, action: action.type.gemAction)
                }
            case .suspiciousAsset: break
            case .tradePerpetuals:
                UIApplication.shared.open(service.deeplinkGemUrl(deeplink: DeepLink.perpetuals.map()).asURL!)
                preferences.isPerpetualEnabled = true
            }
        case let .button(bannerButton):
            switch bannerButton {
            case .buy: onSelectHeader(.buy)
            case .receive: onSelectHeader(.receive)
            }
        case .closeBanner:
            Task {
                try await service.applyBannerAction(key: action.banner.gemKey, action: action.type.gemAction)
            }
        }
        onSelect(url: action.url)
    }

    internal func onSelectEarn() {
        isPresentingSelectedAssetInput.wrappedValue = SelectedAssetInput(
            type: .earn(assetData.asset),
            assetData: assetData,
        )
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
            let enabled = !assetData.isPriceAlertsEnabled
            do {
                try await setPriceAlert(enabled: enabled)
                isPresentingToastMessage = .priceAlert(for: assetData.asset.name, enabled: enabled)
            } catch {
                debugLog("onTogglePriceAlert error \(error)")
            }
        }
    }

    func onSelectTokenStatus() {
        guard let status = statusViewModel?.status else { return }
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
    private var addressExplorerUrl: URL {
        addressLink.url
    }

    private var viewAddressOnTitle: String {
        Localized.Asset.viewAddressOn(addressLink.name)
    }

    private var viewTokenOnTitle: String? {
        if let link = tokenLink {
            return Localized.Asset.viewTokenOn(link.name)
        }
        return .none
    }

    private var tokenExplorerUrl: URL? {
        tokenLink?.url
    }

    private var tokenLink: BlockExplorerLink? {
        guard let tokenId = assetModel.asset.tokenId else {
            return .none
        }
        return service.tokenUrl(chain: assetModel.asset.chain.rawValue, address: tokenId).map { BlockExplorerLink($0) }
    }

    private var addressLink: BlockExplorerLink {
        BlockExplorerLink(service.addressUrl(chain: assetModel.asset.chain.rawValue, address: assetDataModel.address))
    }

    private var stakeBalance: GemAssetBalance {
        GemAssetBalance(assetData.balance, assetId: asset.id)
    }

    private var stakedValue: BigInt {
        BigInt(stakeBalance.stakedValue(chain: asset.chain.rawValue))
    }

    private var feeAssetDataModel: AssetDataViewModel {
        AssetDataViewModel(
            assetData: chainAssetData.feeAssetData,
            formatter: .auto,
            currencyCode: preferences.currency,
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
