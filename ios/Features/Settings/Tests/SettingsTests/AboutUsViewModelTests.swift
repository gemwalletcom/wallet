// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Gemstone
import GemstonePrimitivesTestKit
import GemstoneServices
import GemstoneServicesTestKit
import Primitives
import PrimitivesComponents
@testable import Settings
import SettingsTestKit
import Testing

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

        #expect(model.updateVersion == "99.0.0")
    }

    @Test
    func noReleaseMeansNothingToOffer() async {
        let model = AboutUsViewModel.mock()

        await model.load()

        #expect(model.updateVersion == nil)
    }

    @Test
    func aFailedCheckOffersNothing() async {
        let service = GemAppUpdateServiceMock()
        service.newestError = AnyError("offline")
        let model = AboutUsViewModel.mock(service: service)

        await model.load()

        #expect(model.updateVersion == nil)
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
        #expect(model.contextMenuItems(for: .text(title: .version, value: "1.0 (1)")).count == 2)
        #expect(model.contextMenuItems(for: .loading).isEmpty)
    }

    @Test
    func theVersionReadsAsVersionAndBuild() {
        let model = AboutUsViewModel.mock()

        #expect(model.versionRowValue == "\(Bundle.main.releaseVersionNumber) (\(Bundle.main.buildVersionNumber))")
    }
}

private extension AboutUsViewModel {
    var versionRowValue: String? {
        sections.flatMap(\.values).map(\.row).compactMap { row in
            switch row {
            case let .text(title, value) where title == .version: value
            default: nil
            }
        }.first
    }

    var updateVersion: String? {
        sections.flatMap(\.values).map(\.row).compactMap { row in
            switch row {
            case let .url(title, value, _, _, _) where title == .updateApp: value
            default: nil
            }
        }.first
    }
}
