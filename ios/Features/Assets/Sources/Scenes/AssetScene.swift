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

    public var body: some View {
        List {
            Section {} header: {
                ValueHeaderView(
                    model: model.assetHeaderModel,
                    isPrivacyEnabled: .constant(false),
                    titleActionType: .none,
                    onHeaderAction: model.onSelectHeader,
                    onInfoAction: model.onSelectWalletHeaderInfo,
                )
                .padding(.top, .small)
                .padding(.bottom, .medium)
            }
            .cleanListRow()

            if model.detailsState.showsBanners, let banner = model.visibleBanners.first {
                Section {
                    BannerView(
                        banner: banner,
                        content: model.bannerContent(for: banner),
                        action: model.onSelectBanner,
                    )
                }
                .listRowInsets(.zero)
            }

            if let statusViewModel = model.statusViewModel {
                Section {
                    AssetStatusView(model: statusViewModel, action: model.onSelectTokenStatus)
                }
            }

            if model.detailsState.showsManage {
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

                if model.detailsState.showsPriceAlerts {
                    NavigationLink(
                        value: Scenes.AssetPriceAlert(asset: model.assetData.asset),
                        label: {
                            ListItemView(
                                title: model.priceAlertsViewModel.priceAlertsTitle,
                                subtitle: model.priceAlertsViewModel.priceAlertCount,
                            )
                        },
                    )
                }

                switch model.networkDestination {
                case let .asset(asset):
                    NavigationLink(
                        value: Scenes.Asset(asset: asset.map()),
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
                                title: model.assetDataModel.availableBalanceTitle,
                                subtitle: model.balanceText(value),
                            )
                        case let .staked(value):
                            NavigationCustomLink(
                                with: ListItemView(
                                    title: model.balanceTitle(for: .stake),
                                    subtitle: model.stakeBalanceText(value),
                                ),
                                action: { model.onSelectHeader(.stake) },
                            )
                            .accessibilityIdentifier("stake")
                        case let .earn(value):
                            NavigationCustomLink(
                                with: ListItemView(
                                    title: model.balanceTitle(for: .earn),
                                    subtitle: model.balanceText(value),
                                ),
                                action: { model.onSelectEarn() },
                            )
                            .accessibilityIdentifier("earn")
                        case let .pendingUnconfirmed(value):
                            ListItemView(
                                title: model.assetDataModel.pendingUnconfirmedBalanceTitle,
                                subtitle: model.balanceText(value),
                                infoAction: model.onSelectPendingUnconfirmedInfo,
                            )
                        case let .reserved(value, url):
                            if let url = url.flatMap(URL.init) {
                                SafariNavigationLink(url: url) {
                                    ListItemView(
                                        title: model.assetDataModel.reservedBalanceTitle,
                                        subtitle: model.balanceText(value),
                                    )
                                }
                            } else {
                                ListItemView(
                                    title: model.assetDataModel.reservedBalanceTitle,
                                    subtitle: model.balanceText(value),
                                )
                            }
                        }
                    }
                }
            }

            if model.showEarnButton {
                Section {
                    NavigationCustomLink(
                        with: HStack(spacing: Spacing.medium) {
                            EmojiView(color: Colors.grayVeryLight, emoji: Emoji.WalletAvatar.moneyBag.rawValue)
                                .frame(size: .image.asset)
                            ListItemView(
                                title: model.balanceTitle(for: .earn),
                                subtitle: model.aprModel(for: .earn).text,
                                subtitleStyle: model.aprModel(for: .earn).subtitle.style,
                            )
                        },
                        action: { model.onSelectEarn() },
                    )
                }
            }

            if model.detailsState.showsResources {
                Section(model.resourcesTitle) {
                    ListItemView(field: model.energyField)
                    ListItemView(field: model.bandwidthField)
                }
            }

            if model.showTransactions {
                TransactionsList(
                    model.transactions,
                    currency: model.assetDataModel.currencyCode,
                )
                .listRowInsets(.assetListRowInsets)
            } else {
                Section {
                    Spacer()
                    EmptyContentView(model: model.emptyContentModel)
                        .padding(.bottom, .extraLarge)
                }
                .cleanListRow()
            }
        }
        .refreshableTimer(every: .minutes(5)) { _ in
            await model.load()
        }
        .taskOnce(model.loadOnce)
        .listSectionSpacing(.compact)
        .navigationTitle(model.title)
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
