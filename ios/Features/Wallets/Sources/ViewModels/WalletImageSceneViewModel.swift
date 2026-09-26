// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import struct Gemstone.GemWalletRow
import protocol Gemstone.GemWalletServiceProtocol
import func Gemstone.walletRow
import GemstonePrimitives
import Localization
import Primitives
import PrimitivesComponents
import Store
import Style
import SwiftUI

@MainActor
@Observable
public final class WalletImageSceneViewModel: Sendable {
    struct NFTAssetImageItem: Identifiable {
        let id: String
        let assetImage: AssetImage
    }

    private let service: any GemWalletServiceProtocol

    public let walletQuery: ObservableQuery<WalletQuery>
    public let nftQuery: ObservableQuery<NFTQuery>
    var isPresentingAlertMessage: AlertMessage?

    public var wallet: Wallet {
        walletQuery.value
    }

    public var nftDataList: [NFTData] {
        nftQuery.value
    }

    let emojiViewSize: Sizing = .image.extraLarge
    let emojiList: [EmojiValue] = GemConstants.walletAvatarEmojis.map { EmojiValue(emoji: $0, color: Colors.grayVeryLight) }

    public init(
        wallet: Wallet,
        service: any GemWalletServiceProtocol,
    ) {
        self.service = service
        walletQuery = ObservableQuery(WalletQuery(walletId: wallet.id), initialValue: wallet)
        nftQuery = ObservableQuery(NFTQuery(walletId: wallet.id, filter: .all), initialValue: [])
    }

    var title: String {
        Localized.Common.avatar
    }

    var emptyContentModel: EmptyStateViewModel {
        EmptyStateViewModel(kind: .nfts)
    }

    var row: GemWalletRow {
        walletRow(wallet: wallet.toGem())
    }

    var nftAssetItems: [NFTAssetImageItem] {
        service.avatarItems(data: nftDataList.map { $0.toGem() }).map(\.row).map { row in
            NFTAssetImageItem(
                id: row.id,
                assetImage: AssetImage(
                    type: .text(row.title),
                    imageURL: row.imageUrl.asURL,
                    placeholder: nil,
                    chainPlaceholder: nil,
                ),
            )
        }
    }

    var nftColumns: [GridItem] {
        Array(repeating: GridItem(spacing: .medium), count: 2)
    }

    // MARK: - Public methods

    public func setImage(from url: URL) async {
        do {
            try await service.setImage(url: url, for: wallet)
        } catch {
            isPresentingAlertMessage = AlertMessage(error: error)
        }
    }

    public func onRemoveAvatar() {
        Task {
            do {
                try await service.removeImage(for: wallet)
            } catch {
                isPresentingAlertMessage = AlertMessage(error: error)
            }
        }
    }

    public func setAvatarEmoji(value: EmojiValue) {
        if let image = drawImage(color: value.color.uiColor, text: value.emoji) {
            setImage(image)
        }
    }

    // MARK: - Private methods

    private func drawImage(color: UIColor, text: String) -> UIImage? {
        EmojiAvatarRenderer.image(emoji: text, size: emojiViewSize, color: color)
    }

    private func setImage(_ image: UIImage) {
        guard let data = image.compress() else {
            isPresentingAlertMessage = AlertMessage(message: Localized.Errors.unknown)
            return
        }
        Task {
            do {
                try await service.setImage(data: data, for: wallet)
            } catch {
                isPresentingAlertMessage = AlertMessage(error: error)
            }
        }
    }
}
