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

    @State private var selectedTab: Tab = .emoji
    @State private var model: WalletImageSceneViewModel

    public init(model: WalletImageSceneViewModel) {
        _model = State(initialValue: model)
    }

    public var body: some View {
        let row = model.row
        VStack {
            AvatarView(
                avatarImage: row.avatarImage,
                size: model.emojiViewSize,
                removeAction: row.hasAvatar ? { model.onRemoveAvatar() } : nil,
            )
            .padding(.top, .medium)
            .padding(.bottom, .extraLarge)
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
        .bindQuery(model.walletQuery, model.nftQuery)
        .navigationTitle(model.title)
        .navigationBarTitleDisplayMode(.inline)
        .alertSheet($model.isPresentingAlertMessage)
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
        }
    }

    private var collectionsView: some View {
        let items = model.nftAssetItems
        return ScrollView {
            LazyVGrid(
                columns: model.nftColumns,
                alignment: .center,
                spacing: .medium,
            ) {
                nftAssetListView(items)
            }
            .padding(.horizontal, .medium)
        }
        .overlay {
            if items.isEmpty {
                EmptyContentView(model: model.emptyContentModel)
            }
        }
    }

    private func nftAssetListView(_ items: [WalletImageSceneViewModel.NFTAssetImageItem]) -> some View {
        ForEach(items) { item in
            let view = GridPosterView(model: GridPosterViewModel(assetImage: item.assetImage, title: nil))
            NavigationCustomLink(with: view) {
                onSelectNftAsset(item)
            }
        }
    }
}

// MARK: - Actions

private extension WalletImageScene {
    func onSelectNftAsset(_ item: WalletImageSceneViewModel.NFTAssetImageItem) {
        guard let url = item.assetImage.imageURL else {
            return
        }
        Task {
            await model.setImage(from: url)
        }
    }
}
