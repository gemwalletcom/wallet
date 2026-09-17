// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.GemAboutRow
import protocol Gemstone.GemAppUpdateServiceProtocol
import Components
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

    var sections: [ListSection<AboutRowViewModel>] {
        aboutSections().enumerated().map { index, section in
            ListSection(id: "\(index)", title: nil, image: nil, values: section.rows.map(rowViewModel))
        }
    }

    var title: String {
        Localized.Settings.aboutus
    }

    private func rowViewModel(_ row: GemAboutRow) -> AboutRowViewModel {
        switch row {
        case .termsOfService: AboutRowViewModel(id: String(describing: row), kind: .link(termsOfServiceURL), model: listItem(for: row))
        case .privacyPolicy: AboutRowViewModel(id: String(describing: row), kind: .link(privacyPolicyURL), model: listItem(for: row))
        case .website: AboutRowViewModel(id: String(describing: row), kind: .link(websiteURL), model: listItem(for: row))
        case .community: AboutRowViewModel(id: String(describing: row), kind: .community, model: listItem(for: row))
        case .version: AboutRowViewModel(id: String(describing: row), kind: .version, model: listItem(for: row))
        }
    }

    private func listItem(for row: GemAboutRow) -> ListItemModel {
        switch row {
        case .termsOfService, .privacyPolicy, .website, .community: ListItemModel(title: row.title)
        case .version: ListItemModel(title: row.title, subtitle: versionTextValue)
        }
    }

    var updateListItem: ListItemModel? {
        releaseVersion.map { ListItemModel(title: Localized.UpdateApp.title, subtitle: $0, imageStyle: .settings(assetImage: releaseImage)) }
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
