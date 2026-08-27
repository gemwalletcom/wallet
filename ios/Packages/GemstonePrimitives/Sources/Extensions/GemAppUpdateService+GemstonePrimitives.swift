// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import protocol Gemstone.GemAppUpdateServiceProtocol
import Primitives

public extension GemAppUpdateServiceProtocol {
    func newestRelease() async -> Release? {
        try? await newest(store: PlatformStore.current.json(), currentVersion: Bundle.main.releaseVersionNumber).map { try Release($0) }
    }

    func checkForUpdate() async throws -> Release? {
        try await check(store: PlatformStore.current.json(), currentVersion: Bundle.main.releaseVersionNumber).map { try Release($0) }
    }
}
