// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Primitives
import PrimitivesComponents
import Style
import SwiftUI

struct WalletAssetsSection: View {
    private let model: WalletAssetsSectionViewModel

    init(model: WalletAssetsSectionViewModel) {
        self.model = model
    }

    var body: some View {
        @Bindable var preferences = model.observablePreferences

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
            .listSectionSpacing(.custom(.medium))
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
            ListButton(
                title: model.manageTokenTitle,
                image: model.manageImage,
                action: model.onSelectManage,
            )
            .accessibilityIdentifier("manage")
            .padding(.medium)
            .frame(maxWidth: .infinity, alignment: .center)
        }
        .listRowInsets(.assetListRowInsets)
        .listSectionSpacing(.custom(.medium))
    }
}
