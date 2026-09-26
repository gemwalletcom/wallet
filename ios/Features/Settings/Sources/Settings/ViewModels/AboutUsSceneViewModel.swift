// Copyright (c). Gem Wallet. All rights reserved.

import Components
import func Gemstone.aboutViewState
import struct Gemstone.GemAboutViewState
import protocol Gemstone.GemAppUpdateServiceProtocol
import enum Gemstone.GemListRow
import GemstonePrimitives
import GemstoneServices
import Localization
import Primitives
import PrimitivesComponents
import Style
import SwiftUI

@Observable
@MainActor
public final class AboutUsSceneViewModel: Sendable {
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

    func contextMenuItems(for row: GemListRow, viewState: GemAboutViewState) -> [ContextMenuItemType] {
        switch row {
        case let .text(.version, value):
            [
                .copy(value: value),
                .custom(
                    title: viewState.developerToggle.text,
                    systemImage: SystemImage.info,
                    action: toggleDeveloperMode,
                ),
            ]
        default: []
        }
    }

    var viewState: GemAboutViewState {
        aboutViewState(
            version: Bundle.main.releaseVersionNumber,
            build: String(Bundle.main.buildVersionNumber),
            update: release?.toGem(),
            developerEnabled: preferences.isDeveloperEnabled,
        )
    }
}

extension AboutUsSceneViewModel {
    func toggleDeveloperMode() {
        preferences.isDeveloperEnabled.toggle()
    }

    func load() async {
        release = try? await service.newest(store: PlatformStore.current.toGem(), currentVersion: Bundle.main.releaseVersionNumber)?.toPrimitives()
    }
}
