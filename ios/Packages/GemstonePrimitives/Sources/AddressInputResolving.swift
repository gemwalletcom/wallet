// Copyright (c). Gem Wallet. All rights reserved.

import class Gemstone.GemManageContactService
import class Gemstone.GemNameService
import class Gemstone.GemRecipientService
import struct Gemstone.GemRecipientValidation
import struct Gemstone.GemRecipient
import Primitives

public protocol AddressInputResolving: AnyObject, Sendable {
    func validateRecipient(chain: String, input: String, nameRecord: String?) -> GemRecipientValidation
    func recipient(chain: String, input: String, nameRecord: String?, memo: String?, references: [String]) throws -> GemRecipient
    func isNameSupported(name: String) -> Bool
    func getNameRecord(name: String, chain: String) async throws -> String?
}

public extension AddressInputResolving {
    func getNameRecord(name: String, chain: Primitives.Chain) async throws -> Primitives.NameRecord? {
        try await getNameRecord(name: name, chain: chain.rawValue).map { try Primitives.NameRecord($0) }
    }
}

extension GemNameService: AddressInputResolving {}
extension GemRecipientService: AddressInputResolving {}
extension GemManageContactService: AddressInputResolving {}
