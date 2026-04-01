// Copyright (c). Gem Wallet. All rights reserved.

import Localization
import Primitives
import Style
import SwiftUI
import UIKit

@Observable
@MainActor
public final class AppIconSceneViewModel {
    private var currentIcon: AppIcon

    public init() {
        currentIcon = AppIcon(UIApplication.shared.alternateIconName)
    }

    var title: String { Localized.Settings.Preferences.appIcon }
    var icons: [AppIconViewModel] { AppIcon.allCases.map { AppIconViewModel(icon: $0, isSelected: $0 == currentIcon) } }
    var columns: [GridItem] { Array(repeating: GridItem(.flexible(), spacing: Spacing.medium), count: 3) }
    var supportsAlternateIcons: Bool { UIApplication.shared.supportsAlternateIcons }

    func set(_ icon: AppIcon) async {
        guard icon != currentIcon else { return }
        do {
            currentIcon = icon
            try await UIApplication.shared.setAlternateIconName(icon.iconName)
        } catch {
            debugLog("AppIconSceneViewModel set icon error: \(error)")
        }
    }
}
