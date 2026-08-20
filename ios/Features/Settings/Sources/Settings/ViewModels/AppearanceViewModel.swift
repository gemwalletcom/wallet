// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Localization
import Preferences
import Primitives

@Observable
@MainActor
public final class AppearanceViewModel {
    private let preferences: ObservablePreferences

    public init(preferences: ObservablePreferences = .default) {
        self.preferences = preferences
    }

    var title: String { Localized.Settings.appearanceTitle }
    var options: [Appearance] { Appearance.allCases }
    var appearance: Appearance { preferences.appearance }

    func onSelect(appearance: Appearance) {
        guard appearance != self.appearance else { return }

        preferences.appearance = appearance
    }
}
