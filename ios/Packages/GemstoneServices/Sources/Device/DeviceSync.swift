// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import protocol Gemstone.GemDeviceSync

public final class GemstoneDeviceSync: GemDeviceSync, Sendable {
    private let service: any DeviceServiceable

    public init(service: any DeviceServiceable) {
        self.service = service
    }

    public func syncDevice() async throws {
        try await service.update()
    }
}
