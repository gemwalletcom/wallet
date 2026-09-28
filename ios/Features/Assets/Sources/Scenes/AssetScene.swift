// Copyright (c). Gem Wallet. All rights reserved.

import Components
import enum Gemstone.GemAssetDetailRow
import enum Gemstone.GemAssetNetworkDestination
import enum Gemstone.GemRowAction
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
                    header: model.assetHeader(details),
                    isPrivacyEnabled: .constant(false),
                    titleActionType: .none,
                    onHeaderAction: model.onSelectHeader,
                    onInfoAction: model.onSelectWalletHeaderInfo,
                )
                .padding(.top, .small)
                .padding(.bottom, .medium)
            }
            .cleanListRow()

            if details.state.showsBanners, let banner = details.banner {
                Section {
                    BannerView(
                        row: banner,
                        onDestination: model.onSelectBanner(destination:),
                        onButton: model.onSelectBanner(button:),
                        onClose: model.onCloseBanner,
                    )
                }
                .listRowInsets(.zero)
            }

            if let status = model.verificationStatus(details) {
                Section {
                    AssetStatusView(status: status, action: model.onSelectTokenStatus)
                }
            }

            ForEach(Array(details.sections.enumerated()), id: \.offset) { _, section in
                Section {
                    ForEach(Array(section.rows.enumerated()), id: \.offset) { _, row in
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
        case let .balance(item, action):
            detailLink(action, networkDestination: networkDestination) { ListItemView(model: model.balanceListItem(for: item)) }
                .accessibilityIdentifier(model.accessibilityIdentifier(row) ?? "")
        case let .row(listRow, action):
            detailLink(action, networkDestination: networkDestination) { GemListRowView(row: listRow) }
                .accessibilityIdentifier(model.accessibilityIdentifier(row) ?? "")
        }
    }

    @ViewBuilder
    private func detailLink(_ action: GemRowAction?, networkDestination: GemAssetNetworkDestination?, @ViewBuilder label: @escaping () -> some View) -> some View {
        switch (action, networkDestination) {
        case (.price, _):
            NavigationLink(value: Scenes.Chart(asset: model.asset), label: label)
        case let (.network, .asset(asset)):
            NavigationLink(value: Scenes.Asset(asset: asset.toPrimitives()), label: label)
        case let (.network, .assets(chain)):
            NavigationLink(value: Scenes.NetworkAssets(chain: Chain(core: chain)), label: label)
        case (.stake, _):
            NavigationCustomLink(with: label(), action: model.onSelectStake)
        case (.earn, _):
            NavigationCustomLink(with: label(), action: model.onSelectEarn)
        case let (.explorer(url), _):
            if let url = URL(string: url) {
                SafariNavigationLink(url: url, content: label)
            } else {
                label()
            }
        case (.priceAlerts, _):
            NavigationLink(value: Scenes.AssetPriceAlert(asset: model.assetData.asset), label: label)
        case (.pin, _):
            NavigationCustomLink(with: label(), action: model.onSelectPin)
        case (.addToWallet, _):
            NavigationCustomLink(with: label(), action: model.onSelectEnable)
        default:
            label()
        }
    }
}
