// Copyright (c). Gem Wallet. All rights reserved.

import protocol Gemstone.GemNameServiceProtocol
import GemstonePrimitives
import Primitives
import Store

public struct AddressNameService: Sendable {
    private let addressStore: AddressStore
    private let service: any GemNameServiceProtocol

    public init(
        addressStore: AddressStore,
        service: any GemNameServiceProtocol,
    ) {
        self.addressStore = addressStore
        self.service = service
    }

    public func getAddressName(chain: Chain, address: String) throws -> AddressName? {
        try addressStore.getAddressName(chain: chain, address: address)
    }

    public func getAddressNames(requests: [ChainAddress]) async throws -> [ChainAddress: AddressName] {
        let names = try await service.getAddressNames(requests: requests.map { try $0.json() }).map { try AddressName($0) }
        return Dictionary(uniqueKeysWithValues: names.map { (ChainAddress(chain: $0.chain, address: $0.address), $0) })
    }
}
