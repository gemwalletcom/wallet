// Copyright (c). Gem Wallet. All rights reserved.

import AppService
import Foundation

public extension ConfigUpdateRunner {
    static func mock(
        configService: ConfigService = .mock(),
    ) -> ConfigUpdateRunner {
        ConfigUpdateRunner(configService: configService)
    }
}
