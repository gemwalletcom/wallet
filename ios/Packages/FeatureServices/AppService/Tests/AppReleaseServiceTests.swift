// Copyright (c). Gem Wallet. All rights reserved.

@testable import AppService
import AppServiceTestKit
import Foundation
import Preferences
import PreferencesTestKit
import Primitives
import PrimitivesTestKit
import Testing

struct AppReleaseServiceTests {
    @Test
    func newestRelease() async {
        let configPreferences = ConfigPreferences.mock()
        configPreferences.config = .mock()
        let service = AppReleaseService(configService: ConfigService(configPreferences: configPreferences, service: .mock()))

        #expect(await service.getNewestRelease()?.version == "99.0")
    }

    @Test
    func releaseFromConfig() {
        #expect(AppReleaseService(configService: .mock()).release(ConfigResponse.mock())?.version == "99.0")
    }
}
