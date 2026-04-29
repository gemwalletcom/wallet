// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
@testable import Preferences
import PreferencesTestKit
import Primitives
import Testing

struct ExplorerPreferencesTests {
    private let preferences: any ExplorerPreferencesStorable = ExplorerPreferences.mock()

    @Test
    func defaultPreferences() {
        #expect(preferences.get(chain: .bitcoin) == nil)
    }

    @Test
    func updatePreferences() {
        #expect(preferences.get(chain: .bitcoin) == nil)
        preferences.set(chain: .bitcoin, name: "some name")
        #expect(preferences.get(chain: .bitcoin) == "some name")
    }
}
