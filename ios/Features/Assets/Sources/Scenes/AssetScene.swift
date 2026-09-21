// Copyright (c). Gem Wallet. All rights reserved.

import Components
import struct Gemstone.GemAssetBalanceRow
import enum Gemstone.GemAssetDetailRow
import enum Gemstone.GemAssetNetworkDestination
import GemstonePrimitives
import Localization
import Primitives
import PrimitivesComponents
import Style
import SwiftUI

public struct AssetScene: View {
    private let model: AssetSceneViewModel

    public init(model: AssetSceneViewModel) {
        self.model = model
    }

    @Environment(\.connectionStatus) private var connectionStatus

    public var body: some View {
        let details = model.details
        return List {
            Section {} header: {
                ValueHeaderView(
                    model: model.assetHeaderModel(details),
                    isPrivacyEnabled: .constant(false),
                    titleActionType: .none,
                    onHeaderAction: model.onSelectHeader,
                    onInfoAction: model.onSelectWalletHeaderInfo,
                )
                .padding(.top, .small)
                .padding(.bottom, .medium)
            }
            .cleanListRow()

            if details.state.showsBanners, let banner = model.visibleBanners.first {
                Section {
                    BannerView(
                        model: model.bannerModel(for: banner),
                        action: model.onSelectBanner,
                    )
                }
                .listRowInsets(.zero)
            }

            if let statusViewModel = model.statusViewModel(details) {
                Section {
                    AssetStatusView(model: statusViewModel, action: model.onSelectTokenStatus)
                }
            }

            ForEach(details.sections, id: \.self) { section in
                Section {
                    ForEach(section.rows, id: \.self) { row in
                        detailRow(row, networkDestination: details.networkDestination)
                    }
                } header: {
                    if let title = section.title.text {
                        Text(title)
                    }
                }
            }

            if model.showTransactions {
                TransactionsList(sections: model.transactionSections)
                    .listRowInsets(.assetListRowInsets)
            } else if let error = model.transactionsError {
                Section {
                    ListItemErrorView(errorTitle: Localized.Errors.errorOccurred, error: error)
                }
            } else {
                Section {
                    Spacer()
                    EmptyContentView(model: model.emptyContentModel(details))
                        .padding(.bottom, .extraLarge)
                }
                .cleanListRow()
            }
        }
        .refreshableTimer(every: connectionStatus.refreshInterval(for: .wallet)) { _ in
            await model.refresh()
        }
        .taskOnce(model.loadOnce)
        .listSectionSpacing(.compact)
        .navigationTitle(details.title)
        .contentMargins([.top], .small, for: .scrollContent)
    }
}

// MARK: - UI Components

extension AssetScene {
    @ViewBuilder
    private func detailRow(_ row: GemAssetDetailRow, networkDestination: GemAssetNetworkDestination?) -> some View {
        switch row {
        case let .price(row):
            NavigationLink(
                value: Scenes.Price(asset: model.assetModel.asset),
                label: { ListItemView(title: Localized.Asset.price, subtitle: row.price?.text(), subtitleExtra: row.change?.text(), subtitleStyleExtra: TextStyle(font: .subheadline, color: row.change?.tone.color ?? Colors.gray)) },
            )
            .accessibilityIdentifier("price")
        case let .network(name):
            switch networkDestination {
            case let .asset(asset):
                NavigationLink(
                    value: Scenes.Asset(asset: asset.toPrimitives()),
                    label: { networkView(name: name) },
                )
            case let .assets(chain):
                NavigationLink(
                    value: Scenes.NetworkAssets(chain: Chain(core: chain)),
                    label: { networkView(name: name) },
                )
            case nil:
                networkView(name: name)
            }
        case let .balance(item):
            balanceRow(item)
        case let .earn(row):
            NavigationCustomLink(
                with: GemListRowView(row: row),
                action: { model.onSelectEarn() },
            )
        case let .row(row):
            switch row {
            case .link(.priceAlerts, _, _):
                NavigationLink(
                    value: Scenes.AssetPriceAlert(asset: model.assetData.asset),
                    label: { GemListRowView(row: row) },
                )
            case let .link(title, _, _):
                NavigationCustomLink(with: GemListRowView(row: row)) {
                    model.onSelect(title)
                }
            default:
                GemListRowView(row: row)
            }
        }
    }

    @ViewBuilder
    private func balanceRow(_ item: GemAssetBalanceRow) -> some View {
        switch item.row {
        case .available, .pendingUnconfirmed:
            ListItemView(model: model.balanceListItem(for: item))
        case .staked:
            NavigationCustomLink(
                with: ListItemView(model: model.balanceListItem(for: item)),
                action: { model.onSelectStake() },
            )
            .accessibilityIdentifier("stake")
        case .earn:
            NavigationCustomLink(
                with: ListItemView(model: model.balanceListItem(for: item)),
                action: { model.onSelectEarn() },
            )
            .accessibilityIdentifier("earn")
        case let .reserved(_, url):
            if let url = url.flatMap(URL.init) {
                SafariNavigationLink(url: url) {
                    ListItemView(model: model.balanceListItem(for: item))
                }
            } else {
                ListItemView(model: model.balanceListItem(for: item))
            }
        }
    }

    private func networkView(name: String) -> some View {
        ListItemImageView(
            title: Localized.Transfer.network,
            subtitle: name,
            assetImage: model.networkAssetImage,
            imageSize: .list.image,
        )
    }
}
