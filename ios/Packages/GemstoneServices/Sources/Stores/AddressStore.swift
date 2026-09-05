// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import struct Gemstone.AddressName
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

    public func getAddressName(chain: Gemstone.Chain, address: String) throws -> Gemstone.AddressName? {
        try store.getAddressName(chain: Primitives.Chain(id: chain), address: address).map { $0.map() }
    }

    public func saveAddressNames(names: [Gemstone.AddressName]) async throws {
        try store.updateAddressNames(names.map { $0.map() })
    }

    public func deleteAddressNames(names: [Gemstone.AddressName]) async throws {
        try store.deleteAddressNames(names.map { $0.map() })
    }
}
