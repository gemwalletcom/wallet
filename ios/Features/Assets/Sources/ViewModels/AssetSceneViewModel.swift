// Copyright (c). Gem Wallet. All rights reserved.

import Components
import struct Gemstone.GemAssetBalance
import struct Gemstone.GemAssetBalanceRow
import enum Gemstone.GemAssetDetailRow
import struct Gemstone.GemAssetDetails
import struct Gemstone.GemAssetDetailsInput
import protocol Gemstone.GemAssetDetailsServiceProtocol
import enum Gemstone.GemAssetNetworkDestination
import struct Gemstone.GemBannerRow
import struct Gemstone.GemFormattedNumber
import enum Gemstone.GemHeaderButtonKind
import enum Gemstone.GemListRow
import enum Gemstone.GemListRowTitle
import enum Gemstone.GemLoadState
import enum Gemstone.GemServiceError
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
    public let assetQuery: ObservableQuery<ChainAssetRequest>
    public let bannersQuery: ObservableQuery<BannersRequest>
    public let transactionsQuery: ObservableQuery<MappedRequest<TransactionsRequest, [ListSection<TransactionViewModel>]>>

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

    func detailSections(_ details: GemAssetDetails) -> [AssetDetailSectionItem] {
        details.sections.enumerated().map { index, section in
            AssetDetailSectionItem(
                id: "section-\(index)",
                title: section.title.text,
                rows: section.rows.enumerated().map { rowIndex, row in
                    detailRowItem(row, id: "row-\(index)-\(rowIndex)", networkDestination: details.networkDestination)
                },
            )
        }
    }

    private func detailRowItem(_ row: GemAssetDetailRow, id: String, networkDestination: GemAssetNetworkDestination?) -> AssetDetailRowItem {
        switch row {
        case let .balance(item):
            AssetDetailRowItem(
                id: id,
                content: .item(balanceListItem(for: item)),
                action: balanceAction(item),
                accessibilityIdentifier: balanceAccessibilityIdentifier(item),
            )
        case let .row(row):
            AssetDetailRowItem(
                id: id,
                content: .row(row),
                action: rowAction(row, networkDestination: networkDestination),
                accessibilityIdentifier: accessibilityIdentifier(row),
            )
        }
    }

    private func networkAction(_ destination: GemAssetNetworkDestination?) -> AssetDetailRowAction? {
        switch destination {
        case let .asset(asset): .network(.asset(asset.toPrimitives()))
        case let .assets(chain): .network(.assets(Chain(core: chain)))
        case nil: nil
        }
    }

    private func balanceAction(_ item: GemAssetBalanceRow) -> AssetDetailRowAction? {
        switch item.row {
        case .available, .pendingUnconfirmed: nil
        case .staked: .stake
        case .earn: .earn
        case let .reserved(_, url): url.flatMap(URL.init).map { .explorer($0) }
        }
    }

    private func balanceAccessibilityIdentifier(_ item: GemAssetBalanceRow) -> String? {
        switch item.row {
        case .staked: "stake"
        case .earn: "earn"
        case .available, .pendingUnconfirmed, .reserved: nil
        }
    }

    private func rowAction(_ row: GemListRow, networkDestination: GemAssetNetworkDestination?) -> AssetDetailRowAction? {
        switch row {
        case .quote: .price
        case .network: networkAction(networkDestination)
        case .amount(.stakeApr, _, _), .text(.stakeApr, _): .earn
        case .link(.priceAlerts, _, _): .priceAlerts
        case .link(.pin, _, _), .link(.unpin, _, _): .pin
        case .link(.addToWallet, _, _): .enable
        default: nil
        }
    }

    private func accessibilityIdentifier(_ row: GemListRow) -> String? {
        switch row {
        case .quote: "price"
        default: nil
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
                asset: asset.toGem(),
                ownerAddress: assetDataModel.address,
                metadata: assetData.metadata.toGem(),
                balance: stakeBalance,
                price: assetData.price?.price,
                priceChangePercentage24h: assetData.price?.priceChangePercentage24h,
                currency: preferences.currency.toGem(),
                banners: banners.map { $0.toGem() },
                priceAlerts: assetData.priceAlerts.map { $0.toGem() },
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

    func emptyContentModel(_ details: GemAssetDetails) -> EmptyContentTypeViewModel {
        let state = details.state
        let buy: (() -> Void)? = state.emptyTransactionsAction == .buy ? { self.onSelectBuy() } : nil
        let swap: (() -> Void)? = state.emptyTransactionsAction == .swap ? { self.onSelectSwap() } : nil
        return EmptyContentTypeViewModel(
            type: EmptyContentType(.asset, symbol: assetModel.symbol, isViewOnly: state.isViewOnly, actions: [.buy: buy, .swap: swap]),
        )
    }

    var assetDataModel: AssetDataViewModel {
        AssetDataViewModel(
            assetData: assetData,
            currency: preferences.currency,
        )
    }

    func bannerModel(for row: GemBannerRow) -> BannerViewModel {
        BannerViewModel(row: row)
    }

    func assetHeaderModel(_ details: GemAssetDetails) -> AssetHeaderViewModel {
        AssetHeaderViewModel(assetDataModel: assetDataModel, details: details)
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
                preferences.isPerpetualEnabled = true
                onSelectPerpetuals?()
            case let .url(url):
                onSelect(url: URL(string: url))
            }
        case let .button(bannerButton):
            switch bannerButton {
            case .buy: onSelectHeader(.buy)
            case .receive: onSelectHeader(.receive)
            }
        case .closeBanner:
            Task {
                do {
                    try await service.closeBanner(key: action.key)
                } catch let error as GemServiceError {
                    isPresentingToastMessage = .error(error.text().text)
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
    private var stakeBalance: GemAssetBalance {
        GemAssetBalance(assetData.balance, assetId: asset.id, isActive: assetData.metadata.isActive)
    }

    private func onSelect(url: URL?) {
        guard let url else { return }
        isPresentingAssetSheet = .url(url)
    }

    private func setPriceAlert(enabled: Bool) async throws {
        try await service.setPriceAlert(assetId: assetModel.asset.id.identifier, enabled: enabled)
    }

    func refresh() async {
        let refresh = await service.refresh(assetId: assetModel.asset.id.identifier, hasTransactions: showTransactions)
        transactionsState = refresh.transactions
        for failure in refresh.failures {
            debugLog("asset scene: refresh \(failure.step) failed: \(failure.message)")
        }
    }
}
