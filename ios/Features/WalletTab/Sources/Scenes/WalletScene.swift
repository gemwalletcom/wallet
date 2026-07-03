// Copyright (c). Gem Wallet. All rights reserved.

import Components
import NFT
import Primitives
import PrimitivesComponents
import Style
import SwiftUI

public struct WalletScene: View {
    @State private var model: WalletSceneViewModel

    public init(model: WalletSceneViewModel) {
        _model = State(initialValue: model)
    }

    public var body: some View {
        @Bindable var preferences = model.observablePreferences

        List {
            Section {} header: {
                VStack(spacing: .medium) {
                    ValueHeaderView(
                        model: model.walletHeaderModel,
                        isPrivacyEnabled: $preferences.isHideBalanceEnabled,
                        titleActionType: .privacyToggle,
                        onHeaderAction: model.onHeaderAction,
                        onSubtitleAction: model.onSelectPortfolio,
                        onInfoAction: model.onSelectWatchWalletInfo,
                    )

                    if model.showContentTypePicker {
                        Picker("", selection: $model.selectedContentType) {
                            ForEach(model.availableContentTypes) { type in
                                Text(type.title).tag(type)
                            }
                        }
                        .pickerStyle(.segmented)
                    }
                }
                .padding(.top, .space6)
            }
            .cleanListRow()

            switch model.selectedContentType {
            case .assets:
                if model.showPerpetuals {
                    Section {
                        PerpetualsPreviewView(wallet: model.wallet)
                    } header: {
                        HeaderNavigationLinkView(title: model.perpetualsTitle, destination: Scenes.Perpetuals())
                    }
                    .listRowInsets(.assetListRowInsets)
                    .listSectionSpacing(.custom(.medium))
                }

                if let banner = model.walletBannersModel.allBanners.first {
                    Section {
                        BannerView(
                            banner: banner,
                            action: model.onBanner,
                        )
                    }
                    .listRowInsets(.zero)
                    .listSectionSpacing(.custom(.medium))
                }

                WalletAssetsSection(model: model.assetsModel)
            case .collections:
                NFTCollectionsSection(model: model.collectionsModel)
            case .defi:
                WalletDefiSection()
            }
        }
        .id(model.wallet.id)
        .refreshable {
            await model.refreshSelectedContent()
        }
        .taskOnce {
            Task { await model.fetchOnce() }
        }
    }
}
