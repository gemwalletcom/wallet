// Copyright (c). Gem Wallet. All rights reserved.

import Gemstone
import GemstonePrimitivesTestKit
import GemstoneServices
import GemstoneServicesTestKit
import Primitives
import Testing
@testable import Settings

@MainActor
struct AboutUsViewModelTests {
    private func model(
        service: GemAppUpdateServiceMock = GemAppUpdateServiceMock(),
        preferences: ObservablePreferences = .mock(),
    ) -> AboutUsViewModel {
        AboutUsViewModel(preferences: preferences, service: service)
    }

    @Test
    func theSectionsComeFromCore() {
        let model = model()

        #expect(model.sections.isNotEmpty)
        #expect(model.sections.count > 1)
    }

    @Test
    func aNewerReleaseIsOffered() async {
        let service = GemAppUpdateServiceMock(newest: Gemstone.Release(version: "99.0.0", store: .appStore, upgradeRequired: false))
        let model = model(service: service)

        await model.load()

        #expect(model.releaseVersion == "99.0.0")
    }

    @Test
    func noReleaseMeansNothingToOffer() async {
        let model = model()

        await model.load()

        #expect(model.release == nil)
        #expect(model.releaseVersion == nil)
    }

    @Test
    func aFailedCheckOffersNothing() async {
        let service = GemAppUpdateServiceMock()
        service.newestError = AnyError("offline")
        let model = model(service: service)

        await model.load()

        #expect(model.release == nil)
    }

    @Test
    func theDeveloperMenuItemFollowsThePreference() {
        let preferences = ObservablePreferences.mock()
        preferences.isDeveloperEnabled = false
        let model = model(preferences: preferences)
        let offTitle = model.contextDevTitle

        model.toggleDeveloperMode()

        #expect(preferences.isDeveloperEnabled)
        #expect(model.contextDevTitle != offTitle)
        #expect(model.contextMenuItems.count == 2)
    }

    @Test
    func theVersionReadsAsVersionAndBuild() {
        let model = model()

        #expect(model.versionTextValue.contains("("))
        #expect(model.versionTextValue.hasSuffix(")"))
    }
}
