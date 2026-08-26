// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import class Gemstone.GemConfigService
import GemstonePrimitives
import Preferences
import Primitives

public actor ConfigService {
    private let configPreferences: ConfigPreferences
    private let service: GemConfigService
    private var updateTask: Task<ConfigResponse, Error>?

    public init(
        configPreferences: ConfigPreferences = .standard,
        service: GemConfigService,
    ) {
        self.configPreferences = configPreferences
        self.service = service
    }

    public func updateConfig() async throws {
        if let task = updateTask {
            _ = try await task.value
            return
        }

        updateTask = Task {
            let config = try await ConfigResponse(service.getConfig())
            configPreferences.config = config
            return config
        }

        defer { updateTask = nil }
        _ = try await updateTask?.value
    }

    public func getConfig() async -> ConfigResponse? {
        if let updateTask {
            return try? await updateTask.value
        }
        return configPreferences.config
    }
}
