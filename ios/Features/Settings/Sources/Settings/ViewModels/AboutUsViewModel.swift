// Copyright (c). Gem Wallet. All rights reserved.

import AppService
import Components
import enum Gemstone.SocialUrl
import GemstonePrimitives
import Localization
import Preferences
import Primitives
import PrimitivesComponents
import Style
import SwiftUI

@Observable
@MainActor
public final class AboutUsViewModel: Sendable {
    private let preferences: ObservablePreferences
    private let releaseService: AppReleaseService

    public init(
        preferences: ObservablePreferences,
        releaseService: AppReleaseService,
    ) {
        self.preferences = preferences
        self.releaseService = releaseService
    }

    var title: String {
        Localized.Settings.aboutus
    }

    var termsOfServiceTitle: String {
        Localized.Settings.termsOfServices
    }

    var termsOfServiceURL: URL {
        AppUrl.page(.termsOfService)
    }

    var privacyPolicyTitle: String {
        Localized.Settings.privacyPolicy
    }

    var privacyPolicyURL: URL {
        AppUrl.page(.privacyPolicy)
    }

    var websiteTitle: String {
        Localized.Settings.website
    }

    var websiteURL: URL {
        AppUrl.page(.website)
    }

    var versionTextTitle: String {
        Localized.Settings.version
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

    private let links: [SocialUrl] = [.x, .discord, .telegram, .gitHub, .youTube]
    var linksViewModel: SocialLinksViewModel {
        let assetLinks = links.compactMap {
            if let url = AppUrl.social($0) {
                return AssetLink(
                    name: $0.linkType.rawValue,
                    url: url.absoluteString,
                )
            }
            return .none
        }
        return SocialLinksViewModel(assetLinks: assetLinks)
    }

    var communityTitle: String {
        Localized.Settings.community
    }
}

extension AboutUsViewModel {
    func toggleDeveloperMode() {
        preferences.isDeveloperEnabled.toggle()
    }

    func fetch() async {
        release = await releaseService.getNewestRelease()
    }

    func onUpdate() {
        UIApplication.shared.open(AppUrl.page(.appStore))
    }
}
