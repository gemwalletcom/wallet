// Copyright (c). Gem Wallet. All rights reserved.

import GemstoneServices
import Gemstone

final class DeviceSyncPreflight: GemWalletRequestPreflight, @unchecked Sendable {
    private let deviceService: any GemDeviceServiceProtocol

    init(deviceService: any GemDeviceServiceProtocol) {
        self.deviceService = deviceService
    }

    func prepare() async throws {
        try await deviceService.synchronizeIfNeeded()
    }
}
