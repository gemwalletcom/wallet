// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.GemConfirmFailure
import enum Gemstone.GemConfirmPhase
import struct Gemstone.GemConfirmScreen

public extension GemConfirmScreen {
    static func mock(
        phase: GemConfirmPhase = .loading,
        hasCriticalWarning: Bool = false,
        failure: GemConfirmFailure? = nil,
    ) -> GemConfirmScreen {
        GemConfirmScreen(phase: phase, hasCriticalWarning: hasCriticalWarning, failure: failure)
    }
}
