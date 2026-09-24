// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import struct Gemstone.GemAppUpdateOffer
import protocol Gemstone.GemAppUpdateServiceProtocol
import Primitives

public extension GemAppUpdateServiceProtocol {
    func newestRelease() async -> Release? {
        try? await newest(store: PlatformStore.current.toGem(), currentVersion: Bundle.main.releaseVersionNumber).map { $0.toPrimitives() }
    }

    func checkForUpdate() async throws -> GemAppUpdateOffer? {
        try await check(store: PlatformStore.current.toGem(), currentVersion: Bundle.main.releaseVersionNumber)
    }
}
