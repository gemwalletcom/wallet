// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.Chain
import struct Gemstone.Contact
import struct Gemstone.ContactAddress
import struct Gemstone.GemContactAddressSession
import class Gemstone.GemContactEditorService
import protocol Gemstone.GemContactEditorServiceProtocol
import struct Gemstone.GemContactInput
import struct Gemstone.GemContactScannedAddress
import struct Gemstone.GemContactSession
import class Gemstone.GemPaymentService
import GemstonePrimitivesTestKit

public final class GemContactEditorServiceMock: GemContactEditorServiceProtocol, @unchecked Sendable {
    private let service: GemContactEditorService

    public init() {
        service = GemContactEditorService(
            contacts: .mock(),
            payments: GemPaymentService.mock(),
        )
    }

    public func scannedAddress(input: String) -> GemContactScannedAddress {
        service.scannedAddress(input: input)
    }

    public func saveContact(input: GemContactInput) async throws -> Gemstone.Contact {
        try await service.saveContact(input: input)
    }

    public func newSession(contact: Gemstone.Contact?, addresses: [Gemstone.ContactAddress]) -> GemContactSession {
        service.newSession(contact: contact, addresses: addresses)
    }

    public func newAddressSession(contactId: String, existing: Gemstone.ContactAddress?) -> GemContactAddressSession {
        service.newAddressSession(contactId: contactId, existing: existing)
    }
}
