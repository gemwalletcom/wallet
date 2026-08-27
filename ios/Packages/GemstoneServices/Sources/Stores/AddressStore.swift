// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import typealias Gemstone.AddressName
import typealias Gemstone.Chain
import protocol Gemstone.GemAddressStore
import GemstonePrimitives
import Primitives
import Store

public final class GemstoneAddressStore: GemAddressStore, @unchecked Sendable {
    private let store: AddressStore

    public init(store: AddressStore) {
        self.store = store
    }

    public func getAddressName(chain: Gemstone.Chain, address: String) async throws -> Gemstone.AddressName? {
        try store.getAddressName(chain: Primitives.Chain(id: chain), address: address).map { try $0.json() }
    }

    public func saveAddressNames(names: [Gemstone.AddressName]) async throws {
        try store.updateAddressNames(names.map { try Primitives.AddressName($0) })
    }

    public func deleteAddressNames(names: [Gemstone.AddressName]) async throws {
        try store.deleteAddressNames(names.map { try Primitives.AddressName($0) })
    }
}
