// Copyright (c). Gem Wallet. All rights reserved.

import Gemstone
import GemstonePrimitivesTestKit
import GemstoneServices
import GemstoneServicesTestKit
import Primitives
import Testing
@testable import Settings
import SettingsTestKit

@MainActor
struct AboutUsViewModelTests {
    @Test
    func theSectionsComeFromCore() {
        let model = AboutUsViewModel.mock()

        #expect(model.sections.isNotEmpty)
        #expect(model.sections.count > 1)
    }

    @Test
    func aNewerReleaseIsOffered() async {
        let service = GemAppUpdateServiceMock(newest: Gemstone.Release(version: "99.0.0", store: .appStore, upgradeRequired: false))
        let model = AboutUsViewModel.mock(service: service)

        await model.load()

        #expect(model.releaseVersion == "99.0.0")
    }

    @Test
    func noReleaseMeansNothingToOffer() async {
        let model = AboutUsViewModel.mock()

        await model.load()

        #expect(model.release == nil)
        #expect(model.releaseVersion == nil)
    }

    @Test
    func aFailedCheckOffersNothing() async {
        let service = GemAppUpdateServiceMock()
        service.newestError = AnyError("offline")
        let model = AboutUsViewModel.mock(service: service)

        await model.load()

        #expect(model.release == nil)
    }

    @Test
    func theDeveloperMenuItemFollowsThePreference() {
        let preferences = ObservablePreferences.mock()
        preferences.isDeveloperEnabled = false
        let model = AboutUsViewModel.mock(preferences: preferences)
        let offTitle = model.contextDevTitle

        model.toggleDeveloperMode()

        #expect(preferences.isDeveloperEnabled)
        #expect(model.contextDevTitle != offTitle)
        #expect(model.contextMenuItems.count == 2)
    }

    @Test
    func theVersionReadsAsVersionAndBuild() {
        let model = AboutUsViewModel.mock()

        #expect(model.versionTextValue.contains("("))
        #expect(model.versionTextValue.hasSuffix(")"))
    }
}
