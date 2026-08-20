// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import LocalStore
import Primitives
import Store

public struct AvatarService: Sendable {
    private let store: WalletStore
    private let localStore = LocalStore()

    public init(store: WalletStore) {
        self.store = store
    }

    // MARK: - Store

    public func save(data: Data, for wallet: Wallet) throws {
        let imageUrl = try localStore.store(data, id: UUID().uuidString, documentType: "png")
        if let previous = wallet.imageUrl {
            try localStore.remove(previous)
        }
        try store.setWalletAvatar(wallet.id, path: imageUrl)
    }

    public func save(url: URL, for wallet: Wallet) async throws {
        let (data, _) = try await URLSession.shared.data(from: url)
        try save(data: data, for: wallet)
    }

    public func remove(for wallet: Wallet) throws {
        if let previous = wallet.imageUrl {
            try localStore.remove(previous)
        }
        try store.setWalletAvatar(wallet.id, path: nil)
    }
}
