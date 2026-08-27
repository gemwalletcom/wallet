// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import protocol Gemstone.GemAvatarServiceProtocol
import Primitives

public extension GemAvatarServiceProtocol {
    func setImage(data: Data, for wallet: Wallet) async throws {
        try await setImage(walletId: wallet.id.id, image: data)
    }

    func setImage(url: URL, for wallet: Wallet) async throws {
        try await setImageUrl(walletId: wallet.id.id, url: url.absoluteString)
    }

    func removeImage(for wallet: Wallet) async throws {
        try await removeImage(walletId: wallet.id.id)
    }
}
