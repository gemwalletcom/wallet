// Copyright (c). Gem Wallet. All rights reserved.

internal import Security
import Foundation

struct KeychainError: Error {
    let status: OSStatus
}

extension KeychainError {
    static let conversionError = KeychainError(status: errSecConversionError)
    static let unexpectedError = KeychainError(status: -99999)
}

extension KeychainError: CustomStringConvertible {
    var description: String {
        SecCopyErrorMessageString(status, nil) as String? ?? "OSStatus \(status)"
    }
}

extension KeychainError: CustomNSError {
    static var errorDomain: String { NSOSStatusErrorDomain }

    var errorCode: Int {
        Int(status)
    }

    var errorUserInfo: [String: Any] {
        [NSLocalizedDescriptionKey: description]
    }
}
