// Copyright (c). Gem Wallet. All rights reserved.

import Localization
import GemstonePrimitivesTestKit
import Primitives
@testable import PrimitivesComponents
import PrimitivesTestKit
import Style
import Testing
import struct Gemstone.GemSimulationWarningRow

struct SimulationWarningViewModelTests {
    @Test
    func titleUsesWarningTitleWhenMessageExists() {
        let row = GemSimulationWarningRow.mock(kind: .validationError, message: "Chain ID mismatch")
        let model = SimulationWarningViewModel(row: row)

        #expect(model.id == row)
        #expect(model.title == Localized.Common.warning)
        #expect(model.message == "Chain ID mismatch")
    }

    @Test
    func titleUsesWarningTitleWhenDefaultMessageExists() {
        let model = SimulationWarningViewModel(row: .mock(kind: .unlimitedApproval))

        #expect(model.title == Localized.Simulation.Warning.UnlimitedTokenApproval.title)
        #expect(model.message == Localized.Simulation.Warning.UnlimitedTokenApproval.description)
    }

    @Test
    func colorMatchesSeverity() {
        #expect(SimulationWarningViewModel(row: .mock(kind: .validationError, severity: .critical)).color == Colors.red)
        #expect(SimulationWarningViewModel(row: .mock(kind: .validationError)).color == Colors.orange)
    }

    @Test
    func suspiciousAddressUsesErrorOccurredTitle() {
        let model = SimulationWarningViewModel(row: .mock(kind: .suspiciousSpender, severity: .critical))

        #expect(model.title == Localized.Errors.errorOccurred)
        #expect(model.message == Localized.Common.suspiciousAddress)
    }

    @Test
    func externallyOwnedSpenderUsesSpecificWarningDescription() {
        let model = SimulationWarningViewModel(row: .mock(kind: .externallyOwnedSpender))

        #expect(model.title == Localized.Common.warning)
        #expect(model.message == Localized.Simulation.warningExternallyOwnedSpenderDescription)
    }
}
