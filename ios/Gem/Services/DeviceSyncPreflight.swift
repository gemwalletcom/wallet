// Copyright (c). Gem Wallet. All rights reserved.

import DeviceService
import Gemstone

final class DeviceSyncPreflight: GemWalletRequestPreflight, @unchecked Sendable {
    private let deviceService: any DeviceServiceable

    init(deviceService: any DeviceServiceable) {
        self.deviceService = deviceService
    }

    func prepare() async throws {
        try await deviceService.synchronizeIfNeeded()
    }
}
