// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Keychain

public protocol KeychainPreferenceStorable: Sendable {
    func set(value: String, key: String, accessibility: Accessibility) throws
    func get(key: String) throws -> String?
    func set(value: Data, key: String, accessibility: Accessibility) throws
    func getData(key: String) throws -> Data?
    func remove(key: String) throws
}
