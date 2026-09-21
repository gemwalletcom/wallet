// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.Chain
import struct Gemstone.Contact
import struct Gemstone.ContactAddress
import enum Gemstone.GemAddressFormatStyle
import class Gemstone.GemAddressService
import struct Gemstone.GemContactInput
import struct Gemstone.GemContactScannedAddress
import struct Gemstone.GemContactSession
import class Gemstone.GemContactEditorService
import protocol Gemstone.GemContactEditorServiceProtocol
import class Gemstone.GemPaymentService
import GemstonePrimitivesTestKit

public final class GemContactEditorServiceMock: GemContactEditorServiceProtocol, @unchecked Sendable {
    private let service: GemContactEditorService

    public init() {
        service = GemContactEditorService(
            contacts: .mock(),
            addresses: GemAddressService(),
            payments: GemPaymentService.mock(),
        )
    }

    public func scannedAddress(input: String) -> GemContactScannedAddress {
        service.scannedAddress(input: input)
    }

    public func defaultChain() -> Gemstone.Chain {
        service.defaultChain()
    }

    public func saveContact(input: GemContactInput) async throws -> Gemstone.Contact {
        try await service.saveContact(input: input)
    }

    public func formatAddress(address: String, chain: Gemstone.Chain, style: GemAddressFormatStyle) -> String {
        service.formatAddress(address: address, chain: chain, style: style)
    }

    public func newSession(contact: Gemstone.Contact?, addresses: [Gemstone.ContactAddress]) -> GemContactSession {
        service.newSession(contact: contact, addresses: addresses)
    }
}
