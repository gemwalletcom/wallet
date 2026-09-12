// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Components
import struct Gemstone.GemAssetBalance
import protocol Gemstone.GemAssetDetailsServiceProtocol
import struct Gemstone.GemAssetDetails
import struct Gemstone.GemAssetDetailsInput
import enum Gemstone.GemBalanceRow
import struct Gemstone.GemBannerContent
import struct Gemstone.GemBannerContext
import typealias Gemstone.GemBigUint
import struct Gemstone.GemRecipient
import struct Gemstone.GemTransferData
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

    private var wallet: Wallet {
        walletModel.wallet
    }

    public var title: String {
        details.title
    }

    var priceAlertsTitle: String {
        Localized.Settings.PriceAlerts.title
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

    var balanceRows: [GemBalanceRow] {
        stakeBalance.detailRows(chain: asset.chain.rawValue, isStakeEnabled: assetData.metadata.isStakeEnabled)
    }

    var details: GemAssetDetails {
        service.details(
            input: GemAssetDetailsInput(
                walletType: wallet.type.map(),
                asset: asset.map(),
                ownerAddress: assetDataModel.address,
                metadata: assetData.metadata.map(),
                balance: stakeBalance,
                price: assetData.price?.price,
                bannerEvents: visibleBanners.map { $0.event.map() },
                priceAlerts: assetData.priceAlerts.map { $0.map() },
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

    var showEarnButton: Bool {
        details.state.showsEarn
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
            currencyCode: preferences.currency.rawValue,
        )
    }

    var visibleBanners: [Banner] {
        bannerContext.visibleBanners(stored: banners.map { $0.map() }).map { $0.map() }
    }

    func bannerContent(for banner: Banner) -> GemBannerContent {
        service.bannerContent(event: banner.event.map(), asset: banner.asset?.map())
    }

    private var bannerContext: GemBannerContext {
        GemBannerContext(
            wallet: wallet.map(),
            asset: asset.map(),
            isStakeable: assetData.metadata.isStakeEnabled,
            hasStakeBalance: stakedValue > .zero,
            hasAvailableBalance: assetData.balance.available > 0,
            isAssetActivated: assetData.metadata.isActive,
            assetRankScore: assetData.metadata.rankScore,
            isWalletEmpty: false,
        )
    }

    var assetHeaderModel: AssetHeaderViewModel {
        AssetHeaderViewModel(assetDataModel: assetDataModel, state: details.state)
    }

    public var shareAssetUrl: URL {
        details.shareUrl.asURL!
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
        details.state.priceAlertEnabled ? SystemImage.bellFill : SystemImage.bell
    }

    public var priceAlertsImage: Image {
        Image(systemName: priceAlertsSystemImage)
    }

    public var menuItems: [ActionMenuItemType] {
        let links = details
        return [links.addressLink.map { link in
            .button(title: Localized.Asset.viewAddressOn(link.name), systemImage: SystemImage.globe, action: { self.onSelect(url: link.link.asURL) })
        },
        links.tokenLink.map { link in
            .button(title: Localized.Asset.viewTokenOn(link.name), systemImage: SystemImage.globe, action: { self.onSelect(url: link.link.asURL) })
        },
        .button(title: Localized.Common.share, systemImage: SystemImage.share, action: onSelectShareAsset)].compactMap(\.self)
    }

    var statusViewModel: VerificationStatusViewModel? {
        details.verificationStatus.map { VerificationStatusViewModel(status: $0.map()) }
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
        case .send: .send(.asset(asset: assetData.asset.map()))
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
                UIApplication.shared.open(service.deeplinkGemUrl(deeplink: .perpetuals).asURL!)
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
            let enabled = !details.state.priceAlertEnabled
            do {
                try await setPriceAlert(enabled: enabled)
                isPresentingToastMessage = .priceAlert(for: assetData.asset.name, enabled: enabled)
            } catch {
                isPresentingToastMessage = .error(error.localizedDescription)
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
    private var stakeBalance: GemAssetBalance {
        GemAssetBalance(assetData.balance, assetId: asset.id, isActive: assetData.metadata.isActive)
    }

    private var stakedValue: BigInt {
        BigInt(stakeBalance.stakedValue(chain: asset.chain.rawValue))
    }

    private var feeAssetDataModel: AssetDataViewModel {
        AssetDataViewModel(
            assetData: chainAssetData.feeAssetData,
            formatter: .auto,
            currencyCode: preferences.currency.rawValue,
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
