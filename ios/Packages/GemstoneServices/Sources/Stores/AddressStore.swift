// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import struct Gemstone.AddressName
import typealias Gemstone.Chain
import struct Gemstone.GemAddressNameUpdate
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
        try store.getAddressName(chain: Primitives.Chain(id: chain), address: address).map { $0.toGem() }
    }

    public func saveAddressNames(updates: [GemAddressNameUpdate]) async throws {
        try store.updateAddressNames(
            updates.map { AddressNameUpdate(name: $0.name.toPrimitives(), replacesTypes: $0.replacesTypes.map { $0.toPrimitives() }) },
        )
    }

    public func deleteAddressNames(names: [Gemstone.AddressName]) async throws {
        try store.deleteAddressNames(names.map { $0.toPrimitives() })
    }
}
