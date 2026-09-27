// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives

internal import SwiftUI

public protocol Keystore: Sendable {
    func keystorePassword(createIfMissing: Bool) throws -> String
    /// Migrates pending v3 keystores to v4, reading the password at most once; returns per-wallet failures.
    func migrateV3Keystores(for wallets: [Wallet]) async throws -> [KeystoreMigrationFailure]
    func getPasswordAuthentication() throws -> KeystoreAuthentication
    func destroy() throws
}
