// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import protocol Gemstone.GemConfigServiceProtocol
import GemstonePrimitives
import Primitives

public actor ConfigService {
    private let service: any GemConfigServiceProtocol
    private var updateTask: Task<ConfigResponse, Error>?

    public init(service: any GemConfigServiceProtocol) {
        self.service = service
    }

    public func updateConfig() async throws {
        if let task = updateTask {
            _ = try await task.value
            return
        }
        updateTask = Task {
            try await ConfigResponse(service.updateConfig())
        }
        defer { updateTask = nil }
        _ = try await updateTask?.value
    }

    public func getConfig() async -> ConfigResponse? {
        if let updateTask {
            return try? await updateTask.value
        }
        return try? await ConfigResponse(service.getConfig())
    }
}
