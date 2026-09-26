// Copyright (c). Gem Wallet. All rights reserved.

import Components
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

            ForEach(model.detailSections(details)) { section in
                Section {
                    ForEach(section.rows) { row in
                        detailRow(row)
                    }
                } header: {
                    if let title = section.title {
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
    private func detailRow(_ item: AssetDetailRowItem) -> some View {
        switch item.action {
        case .price:
            NavigationLink(value: Scenes.Chart(asset: model.asset), label: { rowContent(item.content) })
                .accessibilityIdentifier(item.accessibilityIdentifier ?? "")
        case let .network(destination):
            switch destination {
            case let .asset(asset):
                NavigationLink(value: Scenes.Asset(asset: asset), label: { rowContent(item.content) })
            case let .assets(chain):
                NavigationLink(value: Scenes.NetworkAssets(chain: chain), label: { rowContent(item.content) })
            }
        case .stake:
            NavigationCustomLink(with: rowContent(item.content), action: model.onSelectStake)
                .accessibilityIdentifier(item.accessibilityIdentifier ?? "")
        case .earn:
            NavigationCustomLink(with: rowContent(item.content), action: model.onSelectEarn)
                .accessibilityIdentifier(item.accessibilityIdentifier ?? "")
        case let .explorer(url):
            SafariNavigationLink(url: url) { rowContent(item.content) }
        case .priceAlerts:
            NavigationLink(value: Scenes.AssetPriceAlert(asset: model.assetData.asset), label: { rowContent(item.content) })
        case .pin:
            NavigationCustomLink(with: rowContent(item.content), action: model.onSelectPin)
        case .enable:
            NavigationCustomLink(with: rowContent(item.content), action: model.onSelectEnable)
        case .none:
            rowContent(item.content)
        }
    }

    @ViewBuilder
    private func rowContent(_ content: AssetDetailRowContent) -> some View {
        switch content {
        case let .item(model): ListItemView(model: model)
        case let .row(row): GemListRowView(row: row)
        }
    }
}
