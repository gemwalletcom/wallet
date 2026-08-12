// Copyright (c). Gem Wallet. All rights reserved.

import Components
import InfoSheet
import Localization
import NFT
import Primitives
import PrimitivesComponents
import Store
import Style
import SwiftUI

public struct WalletScene: View {
    private var model: WalletSceneViewModel

    public init(model: WalletSceneViewModel) {
        self.model = model
    }

    public var body: some View {
        @Bindable var preferences = model.observablePreferences

        List {
            Section {} header: {
                ValueHeaderView(
                    model: model.walletHeaderModel,
                    isPrivacyEnabled: $preferences.isHideBalanceEnabled,
                    titleActionType: .privacyToggle,
                    onHeaderAction: model.onHeaderAction,
                    onSubtitleAction: model.onSelectPortfolio,
                    onInfoAction: model.onSelectWatchWalletInfo,
                )
                .padding(.top, .space6)
                .padding(.bottom, .space10)
            }
            .cleanListRow()

            if model.showPerpetuals {
                Section {
                    PerpetualsPreviewView(
                        wallet: model.wallet,
                        showBalancePrivacy: $preferences.isHideBalanceEnabled,
                    )
                } header: {
                    HeaderNavigationLinkView(title: model.perpetualsTitle, destination: Scenes.Perpetuals())
                }
                .listRowInsets(.assetListRowInsets)
            }

            if let banner = model.walletBannersModel.allBanners.first {
                Section {
                    BannerView(
                        banner: banner,
                        action: model.onBanner,
                    )
                }
                .listRowInsets(.zero)
            }

            if model.showPinnedSection {
                Section {
                    WalletAssetsList(
                        assets: model.sections.pinned,
                        currencyCode: model.currencyCode,
                        onHideAsset: model.onHideAsset,
                        onPinAsset: model.onPinAsset,
                        onCopyAddress: model.onCopyAddress,
                        showBalancePrivacy: $preferences.isHideBalanceEnabled,
                    )
                } header: {
                    PinnedSectionHeader()
                }
                .listRowInsets(.assetListRowInsets)
            }

            Section {
                WalletAssetsList(
                    assets: model.sections.assets,
                    currencyCode: model.currencyCode,
                    onHideAsset: model.onHideAsset,
                    onPinAsset: model.onPinAsset,
                    onCopyAddress: model.onCopyAddress,
                    showBalancePrivacy: $preferences.isHideBalanceEnabled,
                )
            } header: {
                if model.isLoadingAssets {
                    LoadingTextView(isAnimating: .constant(true))
                        .listRowInsets(.assetListRowInsets)
                        .textCase(nil)
                }
            } footer: {
                if !model.showCollections {
                    manageTokensButton
                }
            }
            .listRowInsets(.assetListRowInsets)

            if model.showCollections {
                Section {
                    CollectionsPreviewView(content: model.collectionsContent)
                } header: {
                    HeaderNavigationLinkView(title: model.collectionsTitle, destination: Scenes.Collections())
                } footer: {
                    manageTokensButton
                }
                .listRowInsets(.assetListRowInsets)
            }
        }
        .listSectionSpacing(.compact)
        .id(model.wallet.id)
        .refreshable {
            await model.fetch()
        }
        .taskOnce {
            Task { await model.fetchOnce() }
        }
        .listSectionSpacing(.compact)
    }
}

// MARK: - UI

extension WalletScene {
    @ViewBuilder
    private var manageTokensButton: some View {
        ListButton(
            title: model.manageTokenTitle,
            image: model.manageImage,
            action: { model.onSelectManage() },
        )
        .accessibilityIdentifier("manage")
        .padding(.medium)
        .frame(maxWidth: .infinity, alignment: .center)
    }
}
