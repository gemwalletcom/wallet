// Copyright (c). Gem Wallet. All rights reserved.

import protocol Gemstone.GemAppUpdateServiceProtocol
import Components
import struct Gemstone.GemAboutSection
import func Gemstone.aboutSections
import func Gemstone.communityLinks
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
    public init(
        preferences: ObservablePreferences,
        service: any GemAppUpdateServiceProtocol,
    ) {
        self.preferences = preferences
        self.service = service
    }

    var sections: [GemAboutSection] {
        aboutSections()
    }

    var title: String {
        Localized.Settings.aboutus
    }


    var termsOfServiceURL: URL {
        AppUrl.page(.termsOfService)
    }


    var privacyPolicyURL: URL {
        AppUrl.page(.privacyPolicy)
    }


    var websiteURL: URL {
        AppUrl.page(.website)
    }


    var versionTextValue: String {
        let version = Bundle.main.releaseVersionNumber
        let number = Bundle.main.buildVersionNumber
        return "\(version) (\(number))"
    }

    var contextDevTitle: String {
        if preferences.isDeveloperEnabled {
            Localized.Settings.disableValue(Localized.Settings.developer)
        } else {
            Localized.Settings.enableValue(Localized.Settings.developer)
        }
    }

    var contextDeveloperImage: String {
        SystemImage.info
    }

    var contextMenuItems: [ContextMenuItemType] {
        [
            .copy(value: versionTextValue),
            .custom(
                title: contextDevTitle,
                systemImage: contextDeveloperImage,
                action: toggleDeveloperMode,
            ),
        ]
    }

    var release: Release?
    var releaseVersion: String? {
        release?.version
    }

    var releaseImage: AssetImage {
        AssetImage.image(Images.Settings.gem)
    }

    var linksViewModel: SocialLinksViewModel {
        SocialLinksViewModel(links: communityLinks())
    }

}

extension AboutUsViewModel {
    func toggleDeveloperMode() {
        preferences.isDeveloperEnabled.toggle()
    }

    func load() async {
        release = await service.newestRelease()
    }

    func onUpdate() {
        UIApplication.shared.open(AppUrl.page(.appStore))
    }
}
