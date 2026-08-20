// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import Localization
import Primitives
import PrimitivesComponents
import Store
import Style
import SwiftUI

public struct WalletImageScene: View {
    enum Tab: Equatable {
        case emoji, collections
    }

    @Environment(\.dismiss) private var dismiss
    @State private var selectedTab: Tab = .emoji
    @State private var model: WalletImageViewModel

    public init(model: WalletImageViewModel) {
        _model = State(initialValue: model)
    }

    public var body: some View {
        VStack {
            AvatarView(
                avatarImage: model.avatarAssetImage(for: model.wallet),
                size: model.emojiViewSize,
                removeAction: model.hasAvatar ? { onRemoveAvatar() } : nil,
            )
            .padding(.top, .medium)
            .padding(.bottom, .extraLarge)
            switch model.source {
            case .onboarding:
                emojiSelector
            case .wallet:
                pickerView
                    .padding(.bottom, .medium)
                    .padding(.horizontal, .medium)
                switch selectedTab {
                case .emoji:
                    emojiSelector
                case .collections:
                    collectionsView
                }
            }
        }
        .bindQuery(model.walletQuery, model.nftQuery)
        .navigationTitle(model.title)
        .navigationBarTitleDisplayMode(.inline)
        .background(Colors.grayBackground)
    }

    private var pickerView: some View {
        Picker("", selection: $selectedTab) {
            Text(Localized.Common.emoji).tag(Tab.emoji)
            Text(Localized.Nft.collections).tag(Tab.collections)
        }
        .pickerStyle(.segmented)
    }

    private var emojiSelector: some View {
        EmojiSelectorView(emojis: model.emojiList) { value in
            model.setAvatarEmoji(value: value)
            onDismiss()
        }
    }

    private var collectionsView: some View {
        ScrollView {
            LazyVGrid(
                columns: model.nftColumns,
                alignment: .center,
                spacing: .medium,
            ) {
                nftAssetListView
            }
            .padding(.horizontal, .medium)
        }
        .overlay {
            if model.nftDataList.isEmpty {
                EmptyContentView(model: model.emptyContentModel)
            }
        }
    }

    private var nftAssetListView: some View {
        ForEach(model.buildNftAssetsItems(from: model.nftDataList)) { item in
            let view = GridPosterView(model: GridPosterViewModel(assetImage: item.assetImage, title: nil))
            NavigationCustomLink(with: view) {
                onSelectNftAsset(item)
            }
        }
    }
}

// MARK: - Actions

private extension WalletImageScene {
    func onRemoveAvatar() {
        model.onRemoveAvatar()
        onDismiss()
    }

    func onSelectNftAsset(_ item: WalletImageViewModel.NFTAssetImageItem) {
        guard let url = item.assetImage.imageURL else {
            return
        }
        Task {
            await model.setImage(from: url)
        }
    }

    func onDismiss() {
        switch model.source {
        case .onboarding: dismiss()
        case .wallet: break
        }
    }
}
