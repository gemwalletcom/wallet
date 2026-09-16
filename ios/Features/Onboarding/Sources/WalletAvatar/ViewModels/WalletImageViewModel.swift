// Copyright (c). Gem Wallet. All rights reserved.

import protocol Gemstone.GemWalletServiceProtocol
import Components
import Foundation
import func Gemstone.walletAvatarEmojis
import func Gemstone.walletRow
import Localization
import Primitives
import PrimitivesComponents
import Store
import Style
import SwiftUI

@MainActor
@Observable
public final class WalletImageViewModel: Sendable {
    public enum Source {
        case onboarding
        case wallet
    }

    struct NFTAssetImageItem: Identifiable {
        let id: String
        let assetImage: AssetImage
    }

    public let source: Source
    private let service: any GemWalletServiceProtocol

    public let walletQuery: ObservableQuery<WalletRequest>
    public let nftQuery: ObservableQuery<NFTRequest>
    var isPresentingAlertMessage: AlertMessage?

    public var wallet: Wallet {
        walletQuery.value
    }

    public var nftDataList: [NFTData] {
        nftQuery.value
    }

    let emojiViewSize: Sizing = .image.extraLarge
    let emojiList: [EmojiValue] = walletAvatarEmojis().map { EmojiValue(emoji: $0, color: Colors.grayVeryLight) }

    public init(
        wallet: Wallet,
        source: Source = .wallet,
        service: any GemWalletServiceProtocol,
    ) {
        self.source = source
        self.service = service
        walletQuery = ObservableQuery(WalletRequest(walletId: wallet.id), initialValue: wallet)
        nftQuery = ObservableQuery(NFTRequest(walletId: wallet.id, filter: .all), initialValue: [])
    }

    var title: String {
        switch source {
        case .onboarding: Localized.Wallet.New.title
        case .wallet: Localized.Common.avatar
        }
    }

    var emptyContentModel: EmptyContentTypeViewModel {
        EmptyContentTypeViewModel(type: .nfts(action: nil))
    }

    var hasAvatar: Bool {
        walletRow(wallet: wallet.toGem()).hasAvatar
    }

    func avatarAssetImage(for wallet: Wallet) -> AssetImage {
        walletRow(wallet: wallet.toGem()).avatarImage
    }

    func buildNftAssetsItems(from list: [NFTData]) -> [NFTAssetImageItem] {
        list
            .map(\.assets)
            .reduce([], +)
            .map {
                NFTAssetImageItem(
                    id: $0.id.identifier,
                    assetImage: AssetImage(
                        type: .text($0.name),
                        imageURL: $0.images.preview.url.asURL,
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
        Task {
            do {
                guard let data = image.compress() else {
                    throw AnyError("Compression image failed")
                }
                try await service.setImage(data: data, for: wallet)
            } catch {
                isPresentingAlertMessage = AlertMessage(error: error)
            }
        }
    }
}
