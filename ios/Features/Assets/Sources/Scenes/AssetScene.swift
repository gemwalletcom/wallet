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
                        banner: banner,
                        content: model.bannerContent(for: banner),
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

            if details.state.showsManage {
                Section(Localized.Common.manage) {
                    NavigationCustomLink(with:
                        ListItemView(
                            title: model.pinText,
                            imageStyle: .list(assetImage: AssetImage(placeholder: model.pinImage)),
                        )) {
                            model.onSelectPin()
                        }
                    NavigationCustomLink(with:
                        ListItemView(
                            title: model.enableText,
                            imageStyle: .list(assetImage: AssetImage(placeholder: model.enableImage)),
                        )) {
                            model.onSelectEnable()
                        }
                }
            }

            Section {
                NavigationLink(
                    value: Scenes.Price(asset: model.assetModel.asset),
                    label: { PriceListItemView(model: model.priceItemViewModel) },
                )
                .accessibilityIdentifier("price")

                if details.state.showsPriceAlerts {
                    NavigationLink(
                        value: Scenes.AssetPriceAlert(asset: model.assetData.asset),
                        label: {
                            ListItemView(
                                title: model.priceAlertsTitle,
                                subtitle: String(details.state.priceAlertsCount),
                            )
                        },
                    )
                }

                switch details.networkDestination {
                case let .asset(asset):
                    NavigationLink(
                        value: Scenes.Asset(asset: asset.toPrimitives()),
                        label: { networkView },
                    )
                case let .assets(chain):
                    NavigationLink(
                        value: Scenes.NetworkAssets(chain: Chain(core: chain)),
                        label: { networkView },
                    )
                case nil:
                    networkView
                }
            }

            if model.balanceRows.isNotEmpty {
                Section(model.balancesTitle) {
                    ForEach(model.balanceRows, id: \.self) { row in
                        switch row {
                        case let .available(value):
                            ListItemView(
                                title: row.title(stakeProvider: .stake),
                                subtitle: model.balanceText(value),
                            )
                        case let .staked(value):
                            NavigationCustomLink(
                                with: ListItemView(
                                    title: row.title(stakeProvider: .stake),
                                    subtitle: model.stakeBalanceText(value),
                                ),
                                action: { model.onSelectStake() },
                            )
                            .accessibilityIdentifier("stake")
                        case let .earn(value):
                            NavigationCustomLink(
                                with: ListItemView(
                                    title: row.title(stakeProvider: .earn),
                                    subtitle: model.balanceText(value),
                                ),
                                action: { model.onSelectEarn() },
                            )
                            .accessibilityIdentifier("earn")
                        case let .pendingUnconfirmed(value):
                            ListItemView(
                                title: row.title(stakeProvider: .stake),
                                subtitle: model.balanceText(value),
                                infoAction: model.onSelectPendingUnconfirmedInfo,
                            )
                        case let .reserved(value, url):
                            if let url = url.flatMap(URL.init) {
                                SafariNavigationLink(url: url) {
                                    ListItemView(
                                        title: row.title(stakeProvider: .stake),
                                        subtitle: model.balanceText(value),
                                    )
                                }
                            } else {
                                ListItemView(
                                    title: row.title(stakeProvider: .stake),
                                    subtitle: model.balanceText(value),
                                )
                            }
                        }
                    }
                }
            }

            if details.state.showsEarn {
                Section {
                    NavigationCustomLink(
                        with: HStack(spacing: Spacing.medium) {
                            EmojiView(color: Colors.grayVeryLight, emoji: Emoji.WalletAvatar.moneyBag.rawValue)
                                .frame(size: .image.asset)
                            ListItemView(
                                title: StakeProviderType.earn.title,
                                subtitle: model.aprModel(for: .earn).text,
                                subtitleStyle: model.aprModel(for: .earn).subtitle.style,
                            )
                        },
                        action: { model.onSelectEarn() },
                    )
                }
            }

            if details.state.showsResources {
                Section(model.resourcesTitle) {
                    ListItemView(field: model.energyField)
                    ListItemView(field: model.bandwidthField)
                }
            }

            if model.showTransactions {
                TransactionsList(sections: model.transactionSections, currency: model.assetDataModel.currency)
                .listRowInsets(.assetListRowInsets)
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
            await model.load()
        }
        .taskOnce(model.loadOnce)
        .listSectionSpacing(.compact)
        .navigationTitle(details.title)
        .contentMargins([.top], .small, for: .scrollContent)
    }
}

// MARK: - UI Components

extension AssetScene {
    private var networkView: some View {
        ListItemImageView(
            title: model.networkField.title.text,
            subtitle: model.networkField.value.text,
            assetImage: model.networkAssetImage,
            imageSize: .list.image,
        )
    }
}
