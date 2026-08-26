// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import protocol Gemstone.GemWalletServiceProtocol
import LocalStore
import Primitives

public struct AvatarService: Sendable {
    private let service: any GemWalletServiceProtocol
    private let localStore = LocalStore()

    public init(service: any GemWalletServiceProtocol) {
        self.service = service
    }

    public func save(data: Data, for wallet: Wallet) async throws {
        let imageUrl = try localStore.store(data, id: UUID().uuidString, documentType: "png")
        if let previous = wallet.imageUrl {
            try localStore.remove(previous)
        }
        try await service.setImageUrl(walletId: wallet.id.id, imageUrl: imageUrl)
    }

    public func save(url: URL, for wallet: Wallet) async throws {
        let (data, _) = try await URLSession.shared.data(from: url)
        try await save(data: data, for: wallet)
    }

    public func remove(for wallet: Wallet) async throws {
        if let previous = wallet.imageUrl {
            try localStore.remove(previous)
        }
        try await service.setImageUrl(walletId: wallet.id.id, imageUrl: nil)
    }
}
