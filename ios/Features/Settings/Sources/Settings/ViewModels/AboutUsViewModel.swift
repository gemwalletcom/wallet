// Copyright (c). Gem Wallet. All rights reserved.

import Components
import enum Gemstone.GemListRow
import protocol Gemstone.GemAppUpdateServiceProtocol
import func Gemstone.aboutSections
import GemstonePrimitives
import Localization
import GemstoneServices
import Primitives
import PrimitivesComponents
import Style
import SwiftUI

@Observable
@MainActor
public final class AboutUsViewModel: Sendable {
    private let preferences: ObservablePreferences
    private let service: any GemAppUpdateServiceProtocol

    private var release: Release?

    public init(
        preferences: ObservablePreferences,
        service: any GemAppUpdateServiceProtocol,
    ) {
        self.preferences = preferences
        self.service = service
    }

    var title: String {
        Localized.Settings.aboutus
    }

    var versionText: String {
        let version = Bundle.main.releaseVersionNumber
        let number = Bundle.main.buildVersionNumber
        return "\(version) (\(number))"
    }

    func contextMenuItems(for row: GemListRow) -> [ContextMenuItemType] {
        switch row {
        case .text(.version, _):
            [
                .copy(value: versionText),
                .custom(
                    title: contextDevTitle,
                    systemImage: SystemImage.info,
                    action: toggleDeveloperMode,
                ),
            ]
        default: []
        }
    }

    var contextDevTitle: String {
        if preferences.isDeveloperEnabled {
            Localized.Settings.disableValue(Localized.Settings.developer)
        } else {
            Localized.Settings.enableValue(Localized.Settings.developer)
        }
    }
}

extension AboutUsViewModel: ListSectionProvideable {
    public var sections: [ListSection<GemListSectionRow>] {
        aboutSections(version: versionText, update: release?.toGem()).listSections
    }
}

extension AboutUsViewModel {
    func toggleDeveloperMode() {
        preferences.isDeveloperEnabled.toggle()
    }

    func load() async {
        release = await service.newestRelease()
    }
}
