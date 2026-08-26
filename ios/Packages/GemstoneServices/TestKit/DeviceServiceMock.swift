// Copyright (c). Gem Wallet. All rights reserved.

import GemstoneServices

public actor DeviceServiceMock: DeviceServiceable {
    public private(set) var updateCalls = 0
    public private(set) var synchronizeIfNeededCalls = 0

    public init() {}

    public func update() async throws {
        updateCalls += 1
    }


    public func synchronizeIfNeeded() async throws {
        synchronizeIfNeededCalls += 1
    }
}
